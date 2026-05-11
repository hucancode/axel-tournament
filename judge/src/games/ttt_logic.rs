// Tic-tac-toe (m,n,k variant) as RoomLogic.
//
// Board is `board_size`x`board_size`; first player to align `win_chain`
// marks horizontally, vertically, or diagonally wins. Defaults: 16x16
// board with 5-in-a-row to win. Player 0 = X, player 1 = O.
//
// Single source of truth: event log. Config travels in the GAME_STARTED
// payload (`<board_size> <win_chain>`) so replays reconstruct identically.

use crate::services::room::logic::RoomLogic;
use crate::services::storage::RoomSnapshot;

const MAX_PLAYERS: usize = 2;
pub const DEFAULT_BOARD_SIZE: u32 = 16;
pub const DEFAULT_WIN_CHAIN: u32 = 5;
const MIN_BOARD_SIZE: u32 = 3;
const MAX_BOARD_SIZE: u32 = 32;
const MIN_WIN_CHAIN: u32 = 3;

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
    pub board_size: u32,
    pub win_chain: u32,
    pub board: Vec<Option<u8>>, // length = board_size * board_size; Some(0)=X, Some(1)=O
    pub turn: u8,                // 0 or 1
    pub winner: Option<u8>,      // None on draw or in-progress
}

impl Default for State {
    fn default() -> Self {
        let size = DEFAULT_BOARD_SIZE as usize;
        Self {
            phase: Phase::Lobby,
            players: Vec::new(),
            host: None,
            board_size: DEFAULT_BOARD_SIZE,
            win_chain: DEFAULT_WIN_CHAIN,
            board: vec![None; size * size],
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

fn parse_config(payload: &str) -> (u32, u32) {
    let mut it = payload.split_whitespace();
    let size = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(DEFAULT_BOARD_SIZE);
    let chain = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(DEFAULT_WIN_CHAIN);
    let size = size.clamp(MIN_BOARD_SIZE, MAX_BOARD_SIZE);
    let chain = chain.clamp(MIN_WIN_CHAIN, size);
    (size, chain)
}

/// Did the move at (row, col) just complete a chain of `win_chain` marks
/// for player `mark`? Scans the four directions through the placed cell.
fn check_winner_at(
    board: &[Option<u8>],
    size: usize,
    chain: usize,
    row: usize,
    col: usize,
    mark: u8,
) -> bool {
    const DIRS: [(i32, i32); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];
    for (dr, dc) in DIRS {
        let mut count = 1;
        for sign in [1i32, -1] {
            let mut r = row as i32 + dr * sign;
            let mut c = col as i32 + dc * sign;
            while r >= 0 && r < size as i32 && c >= 0 && c < size as i32 {
                if board[r as usize * size + c as usize] == Some(mark) {
                    count += 1;
                    if count >= chain {
                        return true;
                    }
                    r += dr * sign;
                    c += dc * sign;
                } else {
                    break;
                }
            }
        }
        if count >= chain {
            return true;
        }
    }
    false
}

fn board_full(board: &[Option<u8>]) -> bool {
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
                let (size, chain) = parse_config(payload);
                state.board_size = size;
                state.win_chain = chain;
                state.board = vec![None; (size * size) as usize];
                state.phase = Phase::Playing;
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
                let size = state.board_size as usize;
                if row < size && col < size {
                    let pos = row * size + col;
                    if state.board[pos].is_none() {
                        state.board[pos] = Some(idx);
                        state.turn = 1 - idx;
                    }
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
                let (size, chain) = parse_config(payload);
                Ok(vec![("GAME_STARTED".into(), format!("{size} {chain}"))])
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
                let size = state.board_size as usize;
                if row >= size || col >= size {
                    return Err("out of range".into());
                }
                let pos = row * size + col;
                if state.board[pos].is_some() {
                    return Err("cell occupied".into());
                }

                let mut out = vec![("MOVE".into(), format!("{player} {row} {col}"))];

                let mut next = state.board.clone();
                next[pos] = Some(idx);
                if check_winner_at(&next, size, state.win_chain as usize, row, col, idx) {
                    out.push(("WINNER".into(), idx.to_string()));
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
    fn x_wins_top_row_3x3() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "3 3");
        drive(&mut s, "a", "MOVE", "0 0");
        drive(&mut s, "b", "MOVE", "1 0");
        drive(&mut s, "a", "MOVE", "0 1");
        drive(&mut s, "b", "MOVE", "1 1");
        drive(&mut s, "a", "MOVE", "0 2");
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, Some(0));
    }

    #[test]
    fn draw_fills_board_3x3() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "3 3");
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
        drive(&mut s, "a", "START", "3 3");
        assert!(Ttt::validate(&s, "b", "MOVE", "0 0").is_err());
    }

    #[test]
    fn rejects_occupied() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "3 3");
        drive(&mut s, "a", "MOVE", "0 0");
        assert!(Ttt::validate(&s, "b", "MOVE", "0 0").is_err());
    }

    #[test]
    fn replay_reconstructs() {
        let events: Vec<(&str, &str)> = vec![
            ("PLAYER_JOINED", "a"),
            ("PLAYER_JOINED", "b"),
            ("GAME_STARTED", "3 3"),
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

    #[test]
    fn default_start_uses_16x16_chain5() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "");
        assert_eq!(s.board_size, 16);
        assert_eq!(s.win_chain, 5);
        assert_eq!(s.board.len(), 16 * 16);
    }

    #[test]
    fn five_in_a_row_diagonal_wins() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "16 5");
        // a builds diagonal (0,0),(1,1),(2,2),(3,3),(4,4); b plays elsewhere.
        for i in 0..5 {
            drive(&mut s, "a", "MOVE", &format!("{i} {i}"));
            if i < 4 {
                drive(&mut s, "b", "MOVE", &format!("0 {}", i + 5));
            }
        }
        assert_eq!(s.phase, Phase::Finished);
        assert_eq!(s.winner, Some(0));
    }

    #[test]
    fn four_in_a_row_does_not_win_when_chain_is_5() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "16 5");
        for i in 0..4 {
            drive(&mut s, "a", "MOVE", &format!("0 {i}"));
            drive(&mut s, "b", "MOVE", &format!("1 {i}"));
        }
        assert_eq!(s.phase, Phase::Playing);
        assert_eq!(s.winner, None);
    }

    #[test]
    fn config_clamps_chain_to_board_size() {
        let mut s = State::default();
        drive(&mut s, "a", "JOIN", "");
        drive(&mut s, "b", "JOIN", "");
        drive(&mut s, "a", "START", "5 99");
        assert_eq!(s.board_size, 5);
        assert_eq!(s.win_chain, 5);
    }
}
