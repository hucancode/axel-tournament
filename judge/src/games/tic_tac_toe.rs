use crate::game_logic::{GameLogic, GameResult};
use crate::player::Player;

#[derive(Clone)]
pub struct TicTacToe;

impl GameLogic for TicTacToe {
    fn new() -> Self {
        TicTacToe
    }

    async fn run(&self, mut players: Vec<Box<dyn Player>>, timeout_ms: u64) -> Vec<GameResult> {
        if players.len() != 2 {
            return vec![GameResult::RuntimeError; players.len()];
        }

        // Set timeout on all players
        for player in &mut players {
            player.set_timeout(timeout_ms);
        }

        let mut board = vec![None; 9];
        let mut current_turn = 0;

        // Send start messages
        let _ = players[0].send_message("START X").await;
        let _ = players[1].send_message("START O").await;

        for _ in 0..9 {
            let move_str = match players[current_turn].receive_message().await {
                Ok(m) => m,
                Err(_) => return if current_turn == 0 { 
                    vec![GameResult::TimeLimitExceeded, GameResult::Accepted(1)] 
                } else { 
                    vec![GameResult::Accepted(1), GameResult::TimeLimitExceeded] 
                }
            };

            let parts: Vec<&str> = move_str.trim().split_whitespace().collect();
            if parts.len() != 3 || parts[0] != "MOVE" {
                return if current_turn == 0 { 
                    vec![GameResult::WrongAnswer, GameResult::Accepted(1)] 
                } else { 
                    vec![GameResult::Accepted(1), GameResult::WrongAnswer] 
                }
            }

            let row: usize = match parts[1].parse() {
                Ok(r) if r < 3 => r,
                _ => return if current_turn == 0 { 
                    vec![GameResult::WrongAnswer, GameResult::Accepted(1)] 
                } else { 
                    vec![GameResult::Accepted(1), GameResult::WrongAnswer] 
                }
            };

            let col: usize = match parts[2].parse() {
                Ok(c) if c < 3 => c,
                _ => return if current_turn == 0 { 
                    vec![GameResult::WrongAnswer, GameResult::Accepted(1)] 
                } else { 
                    vec![GameResult::Accepted(1), GameResult::WrongAnswer] 
                }
            };

            let pos = row * 3 + col;
            if board[pos].is_some() {
                return if current_turn == 0 { 
                    vec![GameResult::WrongAnswer, GameResult::Accepted(1)] 
                } else { 
                    vec![GameResult::Accepted(1), GameResult::WrongAnswer] 
                }
            }

            board[pos] = Some(current_turn);

            // Send board state
            let board_str = self.format_board(&board);
            let _ = players[0].send_message(&board_str).await;
            let _ = players[1].send_message(&board_str).await;

            // Check winner
            if let Some(winner) = self.check_winner(&board) {
                let _ = players[0].send_message(if winner == 0 { "WIN" } else { "LOSE" }).await;
                let _ = players[1].send_message(if winner == 1 { "WIN" } else { "LOSE" }).await;
                return if winner == 0 { 
                    vec![GameResult::Accepted(1), GameResult::Accepted(0)] 
                } else { 
                    vec![GameResult::Accepted(0), GameResult::Accepted(1)] 
                }
            }

            current_turn = 1 - current_turn;
        }

        // Draw
        let _ = players[0].send_message("DRAW").await;
        let _ = players[1].send_message("DRAW").await;
        vec![GameResult::Accepted(0), GameResult::Accepted(0)]
    }

    fn game_id(&self) -> &'static str {
        "tic-tac-toe"
    }

    fn max_players(&self) -> usize {
        2
    }
}

impl TicTacToe {
    fn format_board(&self, board: &[Option<usize>]) -> String {
        let mut result = String::new();
        for row in 0..3 {
            for col in 0..3 {
                let pos = row * 3 + col;
                let symbol = match board[pos] {
                    Some(0) => "X",
                    Some(1) => "O",
                    _ => ".",
                };
                result.push_str(symbol);
            }
            result.push('\n');
        }
        result
    }

    fn check_winner(&self, board: &[Option<usize>]) -> Option<usize> {
        let lines = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // cols
            [0, 4, 8], [2, 4, 6], // diagonals
        ];

        for line in lines {
            if let (Some(a), Some(b), Some(c)) = (board[line[0]], board[line[1]], board[line[2]]) {
                if a == b && b == c {
                    return Some(a);
                }
            }
        }
        None
    }
}
