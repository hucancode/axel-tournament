// Tic-tac-toe playground bot: plays the first empty cell. Deterministic
// for replay parity with the RPS bot. Reads board size from GAME_STARTED.

use crate::games::ttt_logic::{DEFAULT_BOARD_SIZE, DEFAULT_WIN_CHAIN};
use crate::services::playground::PlaygroundStrategy;

#[derive(Default)]
pub struct TttStrategy {
    board: Vec<Option<char>>,
    board_size: usize,
    bot_mark: Option<char>,
    players: Vec<String>,
}

impl PlaygroundStrategy for TttStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !self.players.contains(&pid) {
                    self.players.push(pid);
                }
                None
            }
            "GAME_STARTED" => {
                let mut it = payload.split_whitespace();
                let size = it
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(DEFAULT_BOARD_SIZE) as usize;
                let _chain = it
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(DEFAULT_WIN_CHAIN);
                self.board_size = size;
                self.board = vec![None; size * size];
                self.bot_mark = self
                    .players
                    .iter()
                    .position(|p| p == bot_pid)
                    .map(|i| if i == 0 { 'X' } else { 'O' });
                if self.bot_mark == Some('X') {
                    self.first_empty_move()
                } else {
                    None
                }
            }
            "MOVE" => {
                let mut it = payload.split_whitespace();
                let pid = it.next()?;
                let row: usize = it.next()?.parse().ok()?;
                let col: usize = it.next()?.parse().ok()?;
                let size = self.board_size.max(1);
                let idx = row * size + col;
                let mark = self
                    .players
                    .iter()
                    .position(|p| p == pid)
                    .map(|i| if i == 0 { 'X' } else { 'O' })?;
                if idx < self.board.len() {
                    self.board[idx] = Some(mark);
                }
                if pid != bot_pid {
                    self.first_empty_move()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl TttStrategy {
    fn first_empty_move(&self) -> Option<(String, String)> {
        let idx = self.board.iter().position(|c| c.is_none())?;
        let size = self.board_size.max(1);
        let row = idx / size;
        let col = idx % size;
        Some(("MOVE".into(), format!("{row} {col}")))
    }
}
