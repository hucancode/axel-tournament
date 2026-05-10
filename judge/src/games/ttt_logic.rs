// Tic-tac-toe as RoomLogic.
//
// 3x3 board. Player 0 = X, player 1 = O (assigned by join order).
// Single source of truth: event log.

use crate::services::room::logic::RoomLogic;
use crate::services::storage::RoomSnapshot;

const MAX_PLAYERS: usize = 2;
const CELLS: usize = 9;

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
    pub board: [Option<u8>; CELLS], // Some(0)=X, Some(1)=O
    pub turn: u8,                    // 0 or 1
    pub winner: Option<u8>,          // None on draw or in-progress
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            board: [None; CELLS],
            turn: 0,
            winner: None,
        }
    }
}

impl State {
    fn player_index(&self, player_id: &str) -> Option<u8> {
        self.players.iter().position(|p| p == player_id).map(|i| i as u8)
    }
}

pub struct Ttt;

fn check_winner(board: &[Option<u8>; CELLS]) -> Option<u8> {
    const LINES: [[usize; 3]; 8] = [
        [0, 1, 2], [3, 4, 5], [6, 7, 8],
        [0, 3, 6], [1, 4, 7], [2, 5, 8],
        [0, 4, 8], [2, 4, 6],
    ];
    for l in LINES {
        if let (Some(a), Some(b), Some(c)) = (board[l[0]], board[l[1]], board[l[2]]) {
            if a == b && b == c {
                return Some(a);
            }
        }
    }
    None
}

fn board_full(board: &[Option<u8>; CELLS]) -> bool {
    board.iter().all(|c| c.is_some())
}

impl RoomLogic for Ttt {
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
                state.phase = Phase::Playing;
                state.board = [None; CELLS];
                state.turn = 0;
                state.winner = None;
            }
            "MOVE" => {
                // payload: "<pid> <row> <col>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() != 3 {
                    return;
                }
                let pid = parts[0];
                let (Ok(row), Ok(col)) = (parts[1].parse::<usize>(), parts[2].parse::<usize>())
                else {
                    return;
                };
                let Some(idx) = state.player_index(pid) else { return };
                let pos = row * 3 + col;
                if pos < CELLS && state.board[pos].is_none() {
                    state.board[pos] = Some(idx);
                    state.turn = 1 - idx;
                }
            }
            "WINNER" => {
                if let Ok(w) = payload.trim().parse::<u8>() {
                    state.winner = Some(w);
                    state.phase = Phase::Finished;
                }
            }
            "DRAW" => {
                state.winner = None;
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
                Ok(vec![("GAME_STARTED".into(), String::new())])
            }
            "MOVE" => {
                if state.phase != Phase::Playing {
                    return Err("not playing".into());
                }
                let idx = state.player_index(player).ok_or_else(|| "not in room".to_string())?;
                if state.turn != idx {
                    return Err("not your turn".into());
                }
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() != 2 {
                    return Err("expected: MOVE <row> <col>".into());
                }
                let row: usize = parts[0].parse().map_err(|_| "bad row".to_string())?;
                let col: usize = parts[1].parse().map_err(|_| "bad col".to_string())?;
                if row >= 3 || col >= 3 {
                    return Err("out of range".into());
                }
                let pos = row * 3 + col;
                if state.board[pos].is_some() {
                    return Err("cell occupied".into());
                }

                let mut out = vec![("MOVE".into(), format!("{player} {row} {col}"))];

                // Apply provisionally to detect winner / draw.
                let mut next = state.board;
                next[pos] = Some(idx);
                if let Some(w) = check_winner(&next) {
                    out.push(("WINNER".into(), w.to_string()));
                } else if board_full(&next) {
                    out.push(("DRAW".into(), String::new()));
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
        "tic-tac-toe"
    }

    fn pending_players(state: &Self::State) -> Vec<String> {
        if state.phase != Phase::Playing {
            return Vec::new();
        }
        state
            .players
            .get(state.turn as usize)
            .cloned()
            .into_iter()
            .collect()
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
        Ttt::fold(s, kind, payload);
    }

    fn drive(s: &mut State, player: &str, kind: &str, payload: &str) {
        let evs = Ttt::validate(s, player, kind, payload).expect("validate ok");
        for (k, p) in &evs {
            apply(s, k, p);
        }
    }

    #[test]
    fn x_wins_top_row() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        // a (X) row 0: 0,0; 0,1; 0,2 — interleave with b moves elsewhere
        drive(&mut s, "a", "MOVE", "0 0");
        drive(&mut s, "b", "MOVE", "1 0");
        drive(&mut s, "a", "MOVE", "0 1");
        drive(&mut s, "b", "MOVE", "1 1");
        drive(&mut s, "a", "MOVE", "0 2");
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, Some(0));
    }

    #[test]
    fn draw_fills_board() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        // X O X
        // X O O
        // O X X
        for (player, row, col) in [
            ("a", 0, 0),
            ("b", 0, 1),
            ("a", 0, 2),
            ("b", 1, 1),
            ("a", 1, 0),
            ("b", 1, 2),
            ("a", 2, 1),
            ("b", 2, 0),
            ("a", 2, 2),
        ] {
            drive(&mut s, player, "MOVE", &format!("{row} {col}"));
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, None);
    }

    #[test]
    fn rejects_off_turn() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        assert!(Ttt::validate(&s, "b", "MOVE", "0 0").is_err());
    }

    #[test]
    fn rejects_occupied() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        drive(&mut s, "a", "MOVE", "0 0");
        assert!(Ttt::validate(&s, "b", "MOVE", "0 0").is_err());
    }

    #[test]
    fn replay_reconstructs() {
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", ""),
            ("MOVE", "a 0 0"),
            ("MOVE", "b 1 0"),
            ("MOVE", "a 0 1"),
            ("MOVE", "b 1 1"),
            ("MOVE", "a 0 2"),
            ("WINNER", "0"),
        ];
        let mut s = State::default();
        for (k, p) in &events {
            apply(&mut s, k, p);
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, Some(0));
    }
}
