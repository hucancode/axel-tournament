// Tic-tac-toe playground bot: plays the first empty cell. Deterministic
// for replay parity with the RPS bot.

use crate::services::playground::PlaygroundStrategy;

#[derive(Default)]
pub struct TttStrategy {
    board: [Option<char>; 9],
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
                let idx = row * 3 + col;
                let mark = self
                    .players
                    .iter()
                    .position(|p| p == pid)
                    .map(|i| if i == 0 { 'X' } else { 'O' })?;
                if idx < 9 {
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
        let row = idx / 3;
        let col = idx % 3;
        Some(("MOVE".into(), format!("{row} {col}")))
    }
}
