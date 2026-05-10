// Subprocess transport + match runner for the room wire protocol.
// Spec: judge/protocols/wire.md ("Stdio" transport).
//
// One sandboxed subprocess per bot. The orchestrator writes
// `EVENT seq kind payload\n` lines to stdin and reads `ACT kind [payload]\n`
// lines from stdout. Bots are pre-authorized (no HELLO/WELCOME/since_seq)
// and cannot reconnect.

use crate::protocol::{parse_client, ClientFrame};
use crate::services::room::logic::{LiveRoom, RoomLogic};
use crate::services::sandbox::SandboxedBot;
use crate::services::storage::Event;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub type Bot = Arc<Mutex<SandboxedBot>>;

pub fn wrap(bot: SandboxedBot) -> Bot {
    Arc::new(Mutex::new(bot))
}

async fn send_line(bot: &Mutex<SandboxedBot>, line: &str) -> Result<()> {
    let mut g = bot.lock().await;
    g.stdin.write_all(line.as_bytes()).await?;
    g.stdin.write_all(b"\n").await?;
    g.stdin.flush().await?;
    Ok(())
}

async fn recv_line(bot: &Mutex<SandboxedBot>, timeout: Duration) -> Result<String> {
    let mut g = bot.lock().await;
    let mut line = String::new();
    let n = tokio::time::timeout(timeout, g.stdout.read_line(&mut line))
        .await
        .map_err(|_| anyhow!("bot read timeout"))??;
    if n == 0 {
        return Err(anyhow!("bot eof"));
    }
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}

#[derive(Debug, Clone)]
pub struct MatchOutcome {
    pub scores: Vec<f64>,
    pub faulted_indices: Vec<usize>,
    pub fault_reason: Option<String>,
}

/// Drive a freshly-loaded `LiveRoom<L>` to completion using N bots in
/// JOIN order.
pub async fn run_match<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    bots: Vec<Bot>,
    player_ids: Vec<String>,
    turn_timeout: Duration,
) -> Result<MatchOutcome> {
    if bots.len() != player_ids.len() {
        return Err(anyhow!("bots/player_ids length mismatch"));
    }

    let faults: Arc<Mutex<Vec<Option<String>>>> =
        Arc::new(Mutex::new(vec![None; bots.len()]));

    let mut subscriber = room.subscribe();

    let backlog = room.read_since(0).await?;
    for ev in &backlog {
        let line = format_event(ev);
        for b in &bots {
            let _ = send_line(b, &line).await;
        }
    }

    for pid in &player_ids {
        room.handle_act(pid, "JOIN", "").await.ok();
    }
    if let Some(host) = player_ids.first() {
        room.handle_act(host, "START", "").await.ok();
    }

    let mut readers = Vec::with_capacity(bots.len());
    for (idx, bot) in bots.iter().enumerate() {
        let bot = bot.clone();
        let pid = player_ids[idx].clone();
        let room = room.clone();
        let faults = faults.clone();
        readers.push(tokio::spawn(async move {
            let mark = |reason: String| {
                let faults = faults.clone();
                async move {
                    let mut g = faults.lock().await;
                    if g[idx].is_none() {
                        g[idx] = Some(reason);
                    }
                }
            };
            loop {
                let line = match recv_line(&bot, turn_timeout).await {
                    Ok(l) => l,
                    Err(e) => {
                        let reason = if e.to_string().contains("timeout") {
                            "turn_timeout"
                        } else {
                            "runtime_error"
                        };
                        tracing::debug!("bot {pid} read end ({reason}): {e}");
                        mark(reason.to_string()).await;
                        return;
                    }
                };
                if line.is_empty() {
                    continue;
                }
                match parse_client(&line) {
                    Ok(ClientFrame::Act { kind, payload }) => {
                        if let Err(e) = room.handle_act(&pid, &kind, &payload).await {
                            tracing::debug!("bot {pid} ACT rejected: {e}");
                            mark("illegal_move".to_string()).await;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::debug!("bot {pid} bad frame {line:?}: {e}");
                        mark("malformed_frame".to_string()).await;
                    }
                }
            }
        }));
    }

    let outcome = loop {
        let event = match subscriber.recv().await {
            Ok(e) => e,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("bot match subscriber lagged by {n}; aborting");
                return Err(anyhow!("event stream lagged"));
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                return Err(anyhow!("event stream closed before GAME_END"));
            }
        };
        let line = format_event(&event);
        for b in &bots {
            let _ = send_line(b, &line).await;
        }
        if let Some(scores) = parse_terminal(&event, player_ids.len()) {
            let faults_g = faults.lock().await;
            let faulted_indices: Vec<usize> = faults_g
                .iter()
                .enumerate()
                .filter_map(|(i, r)| r.as_ref().map(|_| i))
                .collect();
            let fault_reason = faulted_indices
                .first()
                .and_then(|i| faults_g[*i].clone());
            break MatchOutcome {
                scores,
                faulted_indices,
                fault_reason,
            };
        }
    };

    for r in readers {
        r.abort();
    }
    Ok(outcome)
}

fn format_event(e: &Event) -> String {
    if e.payload.is_empty() {
        format!("EVENT {} {}", e.seq, e.kind)
    } else {
        format!("EVENT {} {} {}", e.seq, e.kind, e.payload)
    }
}

/// Decode terminal event into per-player scores. Recognises:
/// - `GAME_END <s0> <s1> ...`
/// - `WINNER <idx>`
/// - `DRAW`
pub fn parse_terminal(e: &Event, n_players: usize) -> Option<Vec<f64>> {
    match e.kind.as_str() {
        "GAME_END" => {
            let mut scores: Vec<f64> = e
                .payload
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            scores.resize(n_players, 0.0);
            Some(scores)
        }
        "WINNER" => {
            let mut scores = vec![0.0; n_players];
            if let Ok(idx) = e.payload.trim().parse::<usize>() {
                if idx < n_players {
                    scores[idx] = 1.0;
                }
            }
            Some(scores)
        }
        "DRAW" => Some(vec![0.0; n_players]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::storage::Event;

    #[test]
    fn format_event_with_payload() {
        let e = Event {
            seq: 7,
            kind: "MOVE".into(),
            payload: "alice ROCK".into(),
        };
        assert_eq!(format_event(&e), "EVENT 7 MOVE alice ROCK");
    }

    #[test]
    fn format_event_no_payload() {
        let e = Event {
            seq: 1,
            kind: "GAME_STARTED".into(),
            payload: String::new(),
        };
        assert_eq!(format_event(&e), "EVENT 1 GAME_STARTED");
    }

    #[test]
    fn terminal_game_end_parses_scores() {
        let e = Event {
            seq: 9,
            kind: "GAME_END".into(),
            payload: "3 1".into(),
        };
        let s = parse_terminal(&e, 2).unwrap();
        assert_eq!(s, vec![3.0, 1.0]);
    }

    #[test]
    fn terminal_winner_marks_index() {
        let e = Event {
            seq: 9,
            kind: "WINNER".into(),
            payload: "1".into(),
        };
        let s = parse_terminal(&e, 2).unwrap();
        assert_eq!(s, vec![0.0, 1.0]);
    }

    #[test]
    fn terminal_draw_zero_all() {
        let e = Event {
            seq: 9,
            kind: "DRAW".into(),
            payload: String::new(),
        };
        assert_eq!(parse_terminal(&e, 2).unwrap(), vec![0.0, 0.0]);
    }

    #[test]
    fn non_terminal_returns_none() {
        let e = Event {
            seq: 2,
            kind: "MOVE".into(),
            payload: "alice ROCK".into(),
        };
        assert!(parse_terminal(&e, 2).is_none());
    }
}
