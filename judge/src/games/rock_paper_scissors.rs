use crate::games::{Game, GameResult};
use crate::players::Player;
use rand::Rng;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RockPaperScissors {
    state: Arc<Mutex<GameState>>,
}

#[derive(Debug)]
struct GameState {
    game_started: bool,
    current_round: u32,
    total_rounds: u32,
    scores: [i32; 2],
    game_finished: bool,
    round_history: Vec<[u8; 2]>,
    player_ids: Vec<String>,   // Maps player_id to player number (0 or 1)
}

const ROUNDS: u32 = 5;
const ROUND_VAR: u32 = 2;

impl Game for RockPaperScissors {
    fn new() -> Self {
        RockPaperScissors {
            state: Arc::new(Mutex::new(GameState {
                game_started: false,
                current_round: 0,
                total_rounds: 0,
                scores: [0, 0],
                game_finished: false,
                round_history: Vec::new(),
                player_ids: Vec::new(),
            })),
        }
    }

    async fn run(&self, mut players: Vec<Box<dyn Player>>, timeout_ms: u64) -> Vec<GameResult> {
        if players.len() != 2 {
            return vec![GameResult::RuntimeError; players.len()];
        }

        // Set timeout on all players
        for player in &mut players {
            player.set_timeout(timeout_ms);
        }

        let rounds = rand::rng().random_range((ROUNDS - ROUND_VAR)..=(ROUNDS + ROUND_VAR));

        // Initialize game state
        {
            let mut state = self.state.lock().unwrap();
            state.game_started = true;
            state.current_round = 0;
            state.total_rounds = rounds;
            state.scores = [0, 0];
            state.game_finished = false;
            state.round_history.clear();
            state.player_ids = players.iter().map(|p| p.player_id().to_string()).collect();
        }

        // Send start messages
        let _ = players[0].send_message("START").await;
        let _ = players[1].send_message("START").await;

        for round in 1..=rounds {
            {
                let mut state = self.state.lock().unwrap();
                state.current_round = round;
            }

            let mut moves = Vec::new();

            // Get moves from both players simultaneously
            for i in 0..2 {
                let move_str = match players[i].receive_message().await {
                    Ok(m) => m.trim().to_uppercase(),
                    Err(_) => {
                        let mut state = self.state.lock().unwrap();
                        state.game_finished = true;
                        return if i == 0 {
                            vec![GameResult::TimeLimitExceeded, GameResult::Accepted(state.scores[1])]
                        } else {
                            vec![GameResult::Accepted(state.scores[0]), GameResult::TimeLimitExceeded]
                        }
                    }
                };

                let choice = match move_str.as_str() {
                    "ROCK" => 0,
                    "PAPER" => 1,
                    "SCISSORS" => 2,
                    _ => {
                        let mut state = self.state.lock().unwrap();
                        state.game_finished = true;
                        return if i == 0 {
                            vec![GameResult::WrongAnswer, GameResult::Accepted(state.scores[1])]
                        } else {
                            vec![GameResult::Accepted(state.scores[0]), GameResult::WrongAnswer]
                        }
                    }
                };
                moves.push(choice);
            }

            // Determine winner: 0=rock, 1=paper, 2=scissors
            let winner = match (moves[0], moves[1]) {
                (a, b) if a == b => None, // Draw
                (0, 2) | (1, 0) | (2, 1) => Some(0), // Player 0 wins
                _ => Some(1), // Player 1 wins
            };

            // Update state
            {
                let mut state = self.state.lock().unwrap();
                if let Some(w) = winner {
                    state.scores[w] += 1;
                }

                let _current_scores = state.scores;

                // Record round result (just the moves)
                state.round_history.push([moves[0], moves[1]]);
            }

            let scores = {
                let state = self.state.lock().unwrap();
                state.scores
            };

            // Send round results
            let result_msg = format!("ROUND {} SCORE {} {}", round, scores[0], scores[1]);
            let _ = players[0].send_message(&result_msg).await;
            let _ = players[1].send_message(&result_msg).await;
        }

        // Mark game as finished
        let final_scores = {
            let mut state = self.state.lock().unwrap();
            state.game_finished = true;
            state.scores
        };

        // Send final results
        let _ = players[0].send_message(&format!("SCORE {}", final_scores[0])).await;
        let _ = players[1].send_message(&format!("SCORE {}", final_scores[1])).await;

        // Send END message for graceful exit
        let _ = players[0].send_message("END").await;
        let _ = players[1].send_message("END").await;

        let final_msg = vec![GameResult::Accepted(final_scores[0]), GameResult::Accepted(final_scores[1])];

        // Cleanup players
        drop(players);

        final_msg
    }

    fn game_id(&self) -> &'static str {
        "rock-paper-scissors"
    }

    fn max_players(&self) -> usize {
        2
    }

    fn get_reconnect_state(&self, _player_id: &str) -> Vec<String> {
        let state = self.state.lock().unwrap();

        if !state.game_started {
            return vec![];
        }

        let mut messages = vec![];

        // Send START message
        messages.push("START".to_string());

        // Replay all completed rounds
        // Replay completed rounds
        for (round_idx, moves) in state.round_history.iter().enumerate() {
            let move_str = |m| match m {
                0 => "rock",
                1 => "paper",
                2 => "scissors",
                _ => "unknown"
            };

            messages.push(format!(
                "ROUND {} {} {}",
                round_idx + 1,
                move_str(moves[0]),
                move_str(moves[1])
            ));
        }

        // If game is finished, send final score and END
        if state.game_finished {
            messages.push(format!("SCORE {}", state.scores[0]));
            messages.push("END".to_string());
        } else if state.current_round > 0 {
            // Game is in progress, send current round info
            messages.push(format!(
                "ROUND {} SCORE {} {}",
                state.current_round,
                state.scores[0],
                state.scores[1]
            ));
        }

        messages
    }
}
