// Prisoner's dilemma as RoomLogic.
//
// Iterated PD with fixed round count. Both players move per round; ROUND_RESULT
// emitted when second move arrives.
//
// Payoff matrix (player0, player1):
//   (C,C) = (3,3)
//   (C,D) = (0,5)
//   (D,C) = (5,0)
//   (D,D) = (1,1)

use crate::services::room_logic::RoomLogic;
use crate::services::storage::RoomSnapshot;

const MAX_PLAYERS: usize = 2;
const DEFAULT_ROUNDS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Cooperate,
    Defect,
}

impl Move {
    fn parse(s: &str) -> Option<Move> {
        match s.trim().to_ascii_uppercase().as_str() {
            "C" | "COOPERATE" => Some(Move::Cooperate),
            "D" | "DEFECT" => Some(Move::Defect),
            _ => None,
        }
    }
    fn as_str(&self) -> &'static str {
        match self {
            Move::Cooperate => "C",
            Move::Defect => "D",
        }
    }
}

fn payoff(a: Move, b: Move) -> (i32, i32) {
    match (a, b) {
        (Move::Cooperate, Move::Cooperate) => (3, 3),
        (Move::Cooperate, Move::Defect) => (0, 5),
        (Move::Defect, Move::Cooperate) => (5, 0),
        (Move::Defect, Move::Defect) => (1, 1),
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
    pub players: Vec<String>,
    pub host: Option<String>,
    pub total_rounds: u32,
    pub current_round: u32,
    pub pending_move: [Option<Move>; 2],
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

pub struct Pd;

impl RoomLogic for Pd {
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
                if let Some(idx) = state.players.iter().position(|p| p == pid) {
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
                if let Ok(rounds) = payload.trim().parse::<u32>() {
                    state.total_rounds = rounds;
                }
                state.phase = Phase::Playing;
                state.current_round = 1;
                state.pending_move = [None, None];
                state.scores = [0, 0];
            }
            "MOVE" => {
                // payload: "<pid> <C|D>"
                let mut it = payload.splitn(2, ' ');
                let pid = it.next().unwrap_or("");
                let m = it.next().and_then(Move::parse);
                if let (Some(idx), Some(m)) = (state.player_index(pid), m) {
                    if state.phase == Phase::Playing {
                        state.pending_move[idx] = Some(m);
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
            "CHAT" => {}
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
                if state.players.iter().any(|p| p == player) {
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
                if !state.players.iter().any(|p| p == player) {
                    return Ok(Vec::new());
                }
                let mut out = vec![("PLAYER_LEFT".into(), player.to_string())];
                if state.host.as_deref() == Some(player) {
                    if let Some(h) = state.players.iter().find(|p| p.as_str() != player).cloned() {
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
                let idx = state.player_index(player).ok_or_else(|| "not in room".to_string())?;
                if state.pending_move[idx].is_some() {
                    return Err("already moved this round".into());
                }
                let m = Move::parse(payload).ok_or_else(|| "bad move".to_string())?;
                let mut out = vec![("MOVE".into(), format!("{player} {}", m.as_str()))];

                let other = 1 - idx;
                if let Some(other_m) = state.pending_move[other] {
                    let (m0, m1) = if idx == 0 { (m, other_m) } else { (other_m, m) };
                    let (p0, p1) = payoff(m0, m1);
                    let s = [state.scores[0] + p0, state.scores[1] + p1];
                    out.push((
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
                        out.push(("GAME_END".into(), format!("{} {}", s[0], s[1])));
                    }
                }
                Ok(out)
            }
            _ => Err(format!("unknown action: {kind}")),
        }
    }

    fn max_players() -> usize {
        MAX_PLAYERS
    }

    fn game_id() -> &'static str {
        "prisoners-dilemma"
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

    fn drive(s: &mut State, player: &str, kind: &str, payload: &str) {
        let evs = Pd::validate(s, player, kind, payload).expect("validate ok");
        for (k, p) in &evs {
            Pd::fold(s, k, p);
        }
    }

    #[test]
    fn both_cooperate_three_each() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "1");
        // total_rounds=DEFAULT (10) since validate hardcodes; one round still works
        // Override: simulate single-round game by replaying GAME_STARTED with "1".
        s.total_rounds = 1;
        drive(&mut s, "a", "MOVE", "C");
        drive(&mut s, "b", "MOVE", "C");
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.scores, [3, 3]);
    }

    #[test]
    fn defect_against_cooperate() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        s.total_rounds = 1;
        drive(&mut s, "a", "MOVE", "D");
        drive(&mut s, "b", "MOVE", "C");
        assert_eq!(s.scores, [5, 0]);
        assert_eq!(s.phase, Phase::Finished);
    }

    #[test]
    fn cannot_move_twice_per_round() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        drive(&mut s, "a", "MOVE", "C");
        assert!(Pd::validate(&s, "a", "MOVE", "D").is_err());
    }

    #[test]
    fn replay_reconstructs() {
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", "2"),
            ("MOVE", "a C"),
            ("MOVE", "b D"),
            ("ROUND_RESULT", "1 C D 0 5"),
            ("MOVE", "a D"),
            ("MOVE", "b D"),
            ("ROUND_RESULT", "2 D D 1 6"),
            ("GAME_END", "1 6"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            Pd::fold(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.scores, [1, 6]);
    }
}
