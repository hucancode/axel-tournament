// Subprocess transport for the room wire protocol.
// Spec: judge/protocols/wire.md ("Stdio" transport).
//
// A `BotConn` owns one sandboxed subprocess. The orchestrator writes
// `EVENT seq kind payload\n` lines to its stdin and reads
// `ACT kind [payload]\n` lines from its stdout. Bots are pre-authorized
// (no HELLO/WELCOME/since_seq) and cannot reconnect — they are torn
// down on disconnect. Same parser, same kinds, same RoomLogic as the
// WebSocket handler.

use crate::protocol::{parse_client, ClientFrame};
use crate::services::room::logic::{LiveRoom, RoomLogic};
use crate::services::sandbox::cgroup::CgroupHandle;
use crate::services::sandbox::executor::{fd_to_file, spawn_sandboxed};
use crate::services::storage::Event;
use anyhow::{anyhow, Context, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

/// One sandboxed bot wired to its stdio pipes. Drop kills the subprocess.
pub struct BotConn {
    pid: Option<Pid>,
    stdin: Mutex<TokioFile>,
    stdout: Mutex<BufReader<TokioFile>>,
    _cgroup: CgroupHandle,
}

impl BotConn {
    pub async fn spawn(label: &str, binary_path: &Path) -> Result<Self> {
        let label = label.to_string();
        let path = binary_path.to_path_buf();
        let process = tokio::task::spawn_blocking(move || spawn_sandboxed(&label, &path))
            .await
            .context("spawn task join")??;
        let stdin = TokioFile::from_std(fd_to_file(process.stdin_fd));
        let stdout = BufReader::new(TokioFile::from_std(fd_to_file(process.stdout_fd)));
        Ok(Self {
            pid: Some(process.pid),
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
            _cgroup: process.cgroup,
        })
    }

    /// Write one frame line to bot stdin. The trailing `\n` is added here.
    pub async fn send_line(&self, line: &str) -> Result<()> {
        let mut g = self.stdin.lock().await;
        g.write_all(line.as_bytes()).await?;
        g.write_all(b"\n").await?;
        g.flush().await?;
        Ok(())
    }

    /// Read one frame line from bot stdout, with timeout.
    pub async fn recv_line(&self, timeout: Duration) -> Result<String> {
        let mut g = self.stdout.lock().await;
        let mut line = String::new();
        let n = tokio::time::timeout(timeout, g.read_line(&mut line))
            .await
            .map_err(|_| anyhow!("bot read timeout"))??;
        if n == 0 {
            return Err(anyhow!("bot eof"));
        }
        Ok(line.trim_end_matches(['\n', '\r']).to_string())
    }
}

impl Drop for BotConn {
    fn drop(&mut self) {
        if let Some(pid) = self.pid.take() {
            let _ = kill(pid, Signal::SIGTERM);
            std::thread::sleep(Duration::from_millis(200));
            let _ = kill(pid, Signal::SIGKILL);
        }
    }
}

/// Outcome of a finished AI-vs-AI match.
#[derive(Debug, Clone)]
pub struct MatchOutcome {
    /// Final score per participant, in JOIN order (= participant index).
    pub scores: Vec<f64>,
    /// Indices of participants at fault (runtime error, illegal move,
    /// turn timeout). Empty when the match played out normally.
    pub faulted_indices: Vec<usize>,
    /// Human-readable reason for the fault, if any.
    pub fault_reason: Option<String>,
}

/// Drive a fully-loaded `LiveRoom<L>` to completion using N bots in
/// JOIN order. Each player_id corresponds 1:1 with a `BotConn`.
///
/// Steps (server-driven, no HELLO):
/// 1. JOIN each bot in order. The host (index 0) STARTs.
/// 2. Subscribe to room events; replay all current events to each bot
///    (so bots that need to see PLAYER_JOINED do).
/// 3. Forward live events to all bots.
/// 4. Per bot: read ACT lines, dispatch via `LiveRoom::handle_act`.
/// 5. Resolve when `GAME_END` or `WINNER`/`DRAW` event lands.
pub async fn run_match<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    bots: Vec<Arc<BotConn>>,
    player_ids: Vec<String>,
    turn_timeout: Duration,
) -> Result<MatchOutcome> {
    if bots.len() != player_ids.len() {
        return Err(anyhow!("bots/player_ids length mismatch"));
    }

    // Per-bot fault tracker shared with reader tasks.
    let faults: Arc<Mutex<Vec<Option<String>>>> =
        Arc::new(Mutex::new(vec![None; bots.len()]));

    let mut subscriber = room.subscribe();

    // Replay any committed events to each bot (gap-fill equivalent).
    let backlog = room.read_since(0).await?;
    for ev in &backlog {
        for b in &bots {
            let _ = b.send_line(&format_event(ev)).await;
        }
    }

    // JOIN every bot, then host START.
    for pid in &player_ids {
        room.handle_act(pid, "JOIN", "").await.ok();
    }
    if let Some(host) = player_ids.first() {
        room.handle_act(host, "START", "").await.ok();
    }

    // Per-bot ACT reader task. Records the first fault we see so the
    // orchestrator can attribute losses correctly.
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
                let line = match bot.recv_line(turn_timeout).await {
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

    // Pump events to all bots; finish on terminal kind.
    let outcome = loop {
        let event = match subscriber.recv().await {
            Ok(e) => e,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("AI match subscriber lagged by {n}; aborting");
                return Err(anyhow!("event stream lagged"));
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                return Err(anyhow!("event stream closed before GAME_END"));
            }
        };
        let line = format_event(&event);
        for b in &bots {
            let _ = b.send_line(&line).await;
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

/// Decode a terminal event into per-player scores. Recognises:
/// - `GAME_END <s0> <s1> ...` — space-separated decimals per side
/// - `WINNER <pid_or_idx>`    — winner gets 1.0, others 0.0
/// - `DRAW`                   — all 0.0
fn parse_terminal(e: &Event, n_players: usize) -> Option<Vec<f64>> {
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
