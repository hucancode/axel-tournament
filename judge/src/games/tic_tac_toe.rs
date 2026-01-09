use crate::models::game::{Game, GameResult};
use crate::models::players::Player;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TicTacToe {
    // Shared game state for reconnection support
    state: Arc<Mutex<GameState>>,
}

#[derive(Debug, Clone)]
struct GameState {
    board: Vec<Option<usize>>, // 0 = X, 1 = O
    current_turn: usize,       // 0 or 1
    game_started: bool,
    game_finished: bool,
    winner: Option<usize>,
    player_ids: Vec<String>,   // Maps player_id to player number (0 or 1)
}

impl Game for TicTacToe {
    fn new() -> Self {
        TicTacToe {
            state: Arc::new(Mutex::new(GameState {
                board: vec![None; 9],
                current_turn: 0,
                game_started: false,
                game_finished: false,
                winner: None,
                player_ids: Vec::new(),
            })),
        }
    }

    async fn run(&self, mut players: Vec<Box<dyn Player>>, timeout_ms: u64, game_context: crate::services::room::GameContext) -> Vec<GameResult> {
        if players.len() != 2 {
            return vec![GameResult::RuntimeError; players.len()];
        }

        // Set timeout on all players
        for player in &mut players {
            player.set_timeout(timeout_ms);
        }

        // Initialize game state
        let player_ids: Vec<String> = {
            let mut state = self.state.lock().unwrap();
            state.board = vec![None; 9];
            state.current_turn = 0;
            state.game_started = true;
            state.game_finished = false;
            state.winner = None;
            state.player_ids = players.iter().map(|p| p.player_id().to_string()).collect();
            state.player_ids.clone()
        };

        // Write game initialization to history
        game_context.write_event(&format!("GAME_INIT {} {}", player_ids[0], player_ids[1])).await;

        // Send start messages
        let _ = players[0].send_message("START X").await;
        let _ = players[1].send_message("START O").await;

        // Send initial board state
        let board_msg = self.format_board_message();
        let _ = players[0].send_message(&board_msg).await;
        let _ = players[1].send_message(&board_msg).await;

        for _ in 0..9 {
            let current_turn = {
                let state = self.state.lock().unwrap();
                state.current_turn
            };

            // Send turn notification
            let turn_msg = format!("TURN {}", current_turn);
            let _ = players[0].send_message(&turn_msg).await;
            let _ = players[1].send_message(&turn_msg).await;

            let _ = players[current_turn].send_message("YOUR_TURN").await;

            let move_str = match players[current_turn].receive_message().await {
                Ok(m) => m,
                Err(_) => {
                    let mut state = self.state.lock().unwrap();
                    state.game_finished = true;
                    return if current_turn == 0 {
                        vec![GameResult::TimeLimitExceeded, GameResult::Accepted(1)]
                    } else {
                        vec![GameResult::Accepted(1), GameResult::TimeLimitExceeded]
                    }
                }
            };

            let parts: Vec<&str> = move_str.trim().split_whitespace().collect();
            if parts.len() != 3 || parts[0] != "MOVE" {
                let mut state = self.state.lock().unwrap();
                state.game_finished = true;
                return if current_turn == 0 {
                    vec![GameResult::WrongAnswer, GameResult::Accepted(1)]
                } else {
                    vec![GameResult::Accepted(1), GameResult::WrongAnswer]
                }
            }

            let row: usize = match parts[1].parse() {
                Ok(r) if r < 3 => r,
                _ => {
                    let mut state = self.state.lock().unwrap();
                    state.game_finished = true;
                    return if current_turn == 0 {
                        vec![GameResult::WrongAnswer, GameResult::Accepted(1)]
                    } else {
                        vec![GameResult::Accepted(1), GameResult::WrongAnswer]
                    }
                }
            };

            let col: usize = match parts[2].parse() {
                Ok(c) if c < 3 => c,
                _ => {
                    let mut state = self.state.lock().unwrap();
                    state.game_finished = true;
                    return if current_turn == 0 {
                        vec![GameResult::WrongAnswer, GameResult::Accepted(1)]
                    } else {
                        vec![GameResult::Accepted(1), GameResult::WrongAnswer]
                    }
                }
            };

            let pos = row * 3 + col;

            // Update game state
            let (winner, board_msg) = {
                let mut state = self.state.lock().unwrap();

                if state.board[pos].is_some() {
                    state.game_finished = true;
                    return if current_turn == 0 {
                        vec![GameResult::WrongAnswer, GameResult::Accepted(1)]
                    } else {
                        vec![GameResult::Accepted(1), GameResult::WrongAnswer]
                    }
                }

                state.board[pos] = Some(current_turn);
                state.current_turn = 1 - current_turn;

                let winner = self.check_winner(&state.board);
                if winner.is_some() {
                    state.game_finished = true;
                    state.winner = winner;
                }

                (winner, self.format_board_message_internal(&state.board))
            };

            // Write move to history
            game_context.write_event(&format!("MOVE {} {} {}", current_turn, row, col)).await;

            // Send updated board state
            let _ = players[0].send_message(&board_msg).await;
            let _ = players[1].send_message(&board_msg).await;

            // Check winner
            if let Some(winner) = winner {
                // Write winner to history
                game_context.write_event(&format!("WINNER {}", winner)).await;

                let _ = players[0].send_message(&format!("SCORE {}", if winner == 0 { 1 } else { 0 })).await;
                let _ = players[1].send_message(&format!("SCORE {}", if winner == 1 { 1 } else { 0 })).await;
                let _ = players[0].send_message("END").await;
                let _ = players[1].send_message("END").await;
                return if winner == 0 {
                    vec![GameResult::Accepted(1), GameResult::Accepted(0)]
                } else {
                    vec![GameResult::Accepted(0), GameResult::Accepted(1)]
                }
            }
        }

        // Draw
        {
            let mut state = self.state.lock().unwrap();
            state.game_finished = true;
        }

        // Write draw to history
        game_context.write_event("DRAW").await;

        let _ = players[0].send_message("SCORE 0").await;
        let _ = players[1].send_message("SCORE 0").await;
        let _ = players[0].send_message("END").await;
        let _ = players[1].send_message("END").await;
        vec![GameResult::Accepted(0), GameResult::Accepted(0)]
    }


    fn max_players(&self) -> usize {
        2
    }

    fn get_event_source(&self, player_id: &str) -> Vec<String> {
        let state = self.state.lock().unwrap();

        if !state.game_started {
            return vec![];
        }

        let mut messages = vec![];

        // Determine player number from stored player_ids
        let player_num = state.player_ids.iter().position(|id| id == player_id).unwrap_or(0);
        let symbol = if player_num == 0 { "X" } else { "O" };

        // Send START message
        messages.push(format!("START {}", symbol));

        // Send current board state
        messages.push(self.format_board_message_internal(&state.board));

        // Send current turn
        if !state.game_finished {
            messages.push(format!("TURN {}", state.current_turn));
            if state.current_turn == player_num {
                messages.push("YOUR_TURN".to_string());
            }
        } else if let Some(winner) = state.winner {
            // Game finished, send final score
            let score = if winner == player_num { 1 } else { 0 };
            messages.push(format!("SCORE {}", score));
            messages.push("END".to_string());
        } else {
            // Draw
            messages.push("SCORE 0".to_string());
            messages.push("END".to_string());
        }

        messages
    }

    fn restore_from_events(&self, events: &[String]) {
        let mut state = self.state.lock().unwrap();

        // Reset state
        state.board = vec![None; 9];
        state.current_turn = 0;
        state.game_started = false;
        state.game_finished = false;
        state.winner = None;
        state.player_ids.clear();

        for event in events {
            let parts: Vec<&str> = event.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "GAME_INIT" if parts.len() >= 3 => {
                    state.player_ids = vec![parts[1].to_string(), parts[2].to_string()];
                    state.game_started = true;
                }
                "MOVE" if parts.len() >= 4 => {
                    if let (Ok(player_num), Ok(row), Ok(col)) = (
                        parts[1].parse::<usize>(),
                        parts[2].parse::<usize>(),
                        parts[3].parse::<usize>(),
                    ) {
                        let pos = row * 3 + col;
                        if pos < 9 {
                            state.board[pos] = Some(player_num);
                            state.current_turn = 1 - player_num;
                        }
                    }
                }
                "WINNER" if parts.len() >= 2 => {
                    if let Ok(winner) = parts[1].parse::<usize>() {
                        state.winner = Some(winner);
                        state.game_finished = true;
                    }
                }
                "DRAW" => {
                    state.game_finished = true;
                    state.winner = None;
                }
                _ => {}
            }
        }
    }
}

impl TicTacToe {
    fn format_board_message(&self) -> String {
        let state = self.state.lock().unwrap();
        self.format_board_message_internal(&state.board)
    }

    fn format_board_message_internal(&self, board: &[Option<usize>]) -> String {
        let mut board_str = String::new();
        for row in 0..3 {
            for col in 0..3 {
                let pos = row * 3 + col;
                let symbol = match board[pos] {
                    Some(0) => "X",
                    Some(1) => "O",
                    _ => ".",
                };
                board_str.push_str(symbol);
            }
            if row < 2 {
                board_str.push('\n');
            }
        }
        format!("BOARD {}", board_str.replace('\n', ""))
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
