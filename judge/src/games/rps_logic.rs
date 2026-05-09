// Rock-paper-scissors as RoomLogic.
//
// Single source of truth: the event log. State below is a derived projection
// used for validation and display. It must be reconstructible by replaying
// events from seq=1 in order.

use crate::services::room_logic::RoomLogic;
use crate::services::storage::RoomSnapshot;

const DEFAULT_ROUNDS: u32 = 5;
const MAX_PLAYERS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Rock,
    Paper,
    Scissors,
}

impl Choice {
    fn parse(s: &str) -> Option<Choice> {
        match s.trim().to_ascii_uppercase().as_str() {
            "ROCK" | "R" => Some(Choice::Rock),
            "PAPER" | "P" => Some(Choice::Paper),
            "SCISSORS" | "S" => Some(Choice::Scissors),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Choice::Rock => "ROCK",
            Choice::Paper => "PAPER",
            Choice::Scissors => "SCISSORS",
        }
    }

    /// Returns Some(winner_idx) where 0 means `a` wins, 1 means `b` wins.
    /// None means draw.
    fn winner(a: Choice, b: Choice) -> Option<usize> {
        if a == b {
            return None;
        }
        let a_wins = matches!(
            (a, b),
            (Choice::Rock, Choice::Scissors)
                | (Choice::Paper, Choice::Rock)
                | (Choice::Scissors, Choice::Paper)
        );
        Some(if a_wins { 0 } else { 1 })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    Playing,
    Finished,
}

#[derive(Debug, Clone)]
pub struct State {
    pub phase: Phase,
    pub players: Vec<String>,        // join order; index = side
    pub host: Option<String>,
    pub total_rounds: u32,
    pub current_round: u32,          // 0 before first move
    pub pending_move: [Option<Choice>; 2],
    pub scores: [i32; 2],
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            total_rounds: DEFAULT_ROUNDS,
            current_round: 0,
            pending_move: [None, None],
            scores: [0, 0],
        }
    }
}

impl State {
    fn player_index(&self, player_id: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player_id)
    }
}

pub struct Rps;

impl RoomLogic for Rps {
    type State = State;

    fn fold(state: &mut Self::State, kind: &str, payload: &str) {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !state.players.contains(&pid) && state.players.len() < MAX_PLAYERS {
                    state.players.push(pid.clone());
                    if state.host.is_none() {
                        state.host = Some(pid);
                    }
                }
            }
            "PLAYER_LEFT" => {
                let pid = payload.trim();
                if let Some(idx) = state.player_index(pid) {
                    state.players.remove(idx);
                }
                if state.host.as_deref() == Some(pid) {
                    state.host = state.players.first().cloned();
                }
            }
            "HOST_CHANGED" => {
                state.host = Some(payload.trim().to_string());
            }
            "GAME_STARTED" => {
                if let Some(rounds) = payload.trim().parse::<u32>().ok() {
                    state.total_rounds = rounds;
                }
                state.phase = Phase::Playing;
                state.current_round = 1;
                state.pending_move = [None, None];
                state.scores = [0, 0];
            }
            "MOVE" => {
                // payload: "<pid> <choice>"
                let mut it = payload.splitn(2, ' ');
                let pid = it.next().unwrap_or("");
                let choice = it.next().and_then(Choice::parse);
                if let (Some(idx), Some(c)) = (state.player_index(pid), choice) {
                    if state.phase == Phase::Playing {
                        state.pending_move[idx] = Some(c);
                    }
                }
            }
            "ROUND_RESULT" => {
                // payload: "<round> <m0> <m1> <s0> <s1>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == 5 {
                    if let (Ok(s0), Ok(s1)) = (parts[3].parse::<i32>(), parts[4].parse::<i32>()) {
                        state.scores = [s0, s1];
                    }
                }
                state.pending_move = [None, None];
                if state.current_round < state.total_rounds {
                    state.current_round += 1;
                }
            }
            "GAME_END" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == 2 {
                    if let (Ok(s0), Ok(s1)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        state.scores = [s0, s1];
                    }
                }
                state.phase = Phase::Finished;
            }
            "CHAT" => { /* derived display only; no state mutation */ }
            _ => {}
        }
    }

    fn validate(
        state: &Self::State,
        player: &str,
        kind: &str,
        payload: &str,
    ) -> Result<Vec<(String, String)>, String> {
        match kind {
            "JOIN" => {
                if state.players.contains(&player.to_string()) {
                    return Ok(Vec::new());
                }
                if state.phase != Phase::Lobby {
                    return Err("game already started".into());
                }
                if state.players.len() >= MAX_PLAYERS {
                    return Err("room full".into());
                }
                Ok(vec![("PLAYER_JOINED".into(), player.to_string())])
            }
            "LEAVE" => {
                if !state.players.contains(&player.to_string()) {
                    return Ok(Vec::new());
                }
                let mut out = vec![("PLAYER_LEFT".into(), player.to_string())];
                // Host transfer if leaver was host and others remain.
                if state.host.as_deref() == Some(player) {
                    let next_host = state
                        .players
                        .iter()
                        .find(|p| p.as_str() != player)
                        .cloned();
                    if let Some(h) = next_host {
                        out.push(("HOST_CHANGED".into(), h));
                    }
                }
                Ok(out)
            }
            "CHAT" => {
                let msg = sanitize(payload, 500);
                if msg.is_empty() {
                    return Ok(Vec::new());
                }
                Ok(vec![("CHAT".into(), format!("{player} {msg}"))])
            }
            "START" => {
                if state.host.as_deref() != Some(player) {
                    return Err("only host may start".into());
                }
                if state.phase != Phase::Lobby {
                    return Err("already started".into());
                }
                if state.players.len() < 2 {
                    return Err("need 2 players".into());
                }
                Ok(vec![("GAME_STARTED".into(), DEFAULT_ROUNDS.to_string())])
            }
            "MOVE" => {
                if state.phase != Phase::Playing {
                    return Err("not playing".into());
                }
                let idx = state
                    .player_index(player)
                    .ok_or_else(|| "not in room".to_string())?;
                if state.pending_move[idx].is_some() {
                    return Err("already moved this round".into());
                }
                let choice = Choice::parse(payload).ok_or_else(|| "bad choice".to_string())?;

                let mut events = vec![(
                    "MOVE".to_string(),
                    format!("{player} {}", choice.as_str()),
                )];

                // Did this complete the round?
                let other = 1 - idx;
                if let Some(other_choice) = state.pending_move[other] {
                    let (m0, m1) = if idx == 0 {
                        (choice, other_choice)
                    } else {
                        (other_choice, choice)
                    };
                    let mut s = state.scores;
                    if let Some(w) = Choice::winner(m0, m1) {
                        s[w] += 1;
                    }
                    events.push((
                        "ROUND_RESULT".into(),
                        format!(
                            "{} {} {} {} {}",
                            state.current_round,
                            m0.as_str(),
                            m1.as_str(),
                            s[0],
                            s[1]
                        ),
                    ));
                    if state.current_round >= state.total_rounds {
                        events.push(("GAME_END".into(), format!("{} {}", s[0], s[1])));
                    }
                }
                Ok(events)
            }
            _ => Err(format!("unknown action: {kind}")),
        }
    }

    fn max_players() -> usize {
        MAX_PLAYERS
    }

    fn game_id() -> &'static str {
        "rock-paper-scissors"
    }

    fn snapshot(state: &Self::State) -> RoomSnapshot {
        RoomSnapshot {
            phase: phase_str(&state.phase),
            host: state.host.clone(),
            players: state.players.clone(),
        }
    }
}

fn phase_str(phase: &Phase) -> String {
    match phase {
        Phase::Lobby => "lobby",
        Phase::Playing => "playing",
        Phase::Finished => "finished",
    }
    .to_string()
}

fn sanitize(s: &str, max_len: usize) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .take(max_len)
        .collect();
    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(s: &mut State, kind: &str, payload: &str) {
        Rps::fold(s, kind, payload);
    }

    #[test]
    fn join_first_player_becomes_host() {
        let mut s = State::default();
        apply(&mut s, "PLAYER_JOINED", "alice");
        assert_eq!(s.players, vec!["alice"]);
        assert_eq!(s.host.as_deref(), Some("alice"));
    }

    #[test]
    fn full_game_rock_beats_scissors() {
        let mut s = State::default();
        for kind_payload in [
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", "1"),
        ] {
            apply(&mut s, kind_payload.0, kind_payload.1);
        }
        assert_eq!(s.phase, Phase::Playing);
        // a moves rock; validate as second mover (b) finishes round.
        let evs = Rps::validate(&s, "a", "MOVE", "ROCK").unwrap();
        for (k, p) in &evs {
            apply(&mut s, k, p);
        }
        let evs2 = Rps::validate(&s, "b", "MOVE", "SCISSORS").unwrap();
        for (k, p) in &evs2 {
            apply(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.scores, [1, 0]);
    }

    #[test]
    fn host_transfers_on_leave() {
        let mut s = State::default();
        apply(&mut s, "PLAYER_JOINED", "alice");
        apply(&mut s, "PLAYER_JOINED", "bob");
        let evs = Rps::validate(&s, "alice", "LEAVE", "").unwrap();
        for (k, p) in &evs {
            apply(&mut s, k, p);
        }
        assert_eq!(s.host.as_deref(), Some("bob"));
        assert_eq!(s.players, vec!["bob"]);
    }

    #[test]
    fn cannot_move_twice_per_round() {
        let mut s = State::default();
        apply(&mut s, "PLAYER_JOINED", "a");
        apply(&mut s, "PLAYER_JOINED", "b");
        apply(&mut s, "GAME_STARTED", "3");
        let evs = Rps::validate(&s, "a", "MOVE", "ROCK").unwrap();
        for (k, p) in &evs {
            apply(&mut s, k, p);
        }
        assert!(Rps::validate(&s, "a", "MOVE", "PAPER").is_err());
    }

    #[test]
    fn replay_reconstructs_state() {
        // Capture an event sequence.
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", "2"),
            ("MOVE", "a ROCK"),
            ("MOVE", "b PAPER"),
            ("ROUND_RESULT", "1 ROCK PAPER 0 1"),
            ("MOVE", "a SCISSORS"),
            ("MOVE", "b ROCK"),
            ("ROUND_RESULT", "2 SCISSORS ROCK 0 2"),
            ("GAME_END", "0 2"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            apply(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.scores, [0, 2]);
        assert_eq!(s.players, vec!["a", "b"]);
    }
}
