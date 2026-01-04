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
        tracing::info!("RPS: Starting game with {} players, timeout: {}ms", players.len(), timeout_ms);
        
        if players.len() != 2 {
            tracing::error!("RPS: Invalid player count: {}", players.len());
            return vec![GameResult::RuntimeError; players.len()];
        }

        // Set timeout on all players
        for (i, player) in players.iter_mut().enumerate() {
            player.set_timeout(timeout_ms);
            tracing::debug!("RPS: Set timeout for player {}: {}", i, player.player_id());
        }

        let rounds = rand::rng().random_range((ROUNDS - ROUND_VAR)..=(ROUNDS + ROUND_VAR));
        tracing::info!("RPS: Game will have {} rounds", rounds);

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
            tracing::debug!("RPS: Initialized game state - players: {:?}", state.player_ids);
        }

        // Send start messages
        tracing::debug!("RPS: Sending START messages to both players");
        let start_result_0 = players[0].send_message("START").await;
        let start_result_1 = players[1].send_message("START").await;
        tracing::debug!("RPS: START message results - P0: {:?}, P1: {:?}", start_result_0, start_result_1);

        for round in 1..=rounds {
            tracing::info!("RPS: Starting round {}/{}", round, rounds);
            {
                let mut state = self.state.lock().unwrap();
                state.current_round = round;
            }

            let mut moves = Vec::new();

            // Get moves from both players simultaneously
            for i in 0..2 {
                tracing::debug!("RPS: Waiting for move from player {} ({})", i, players[i].player_id());
                let move_str = match players[i].receive_message().await {
                    Ok(m) => {
                        tracing::debug!("RPS: Player {} sent move: '{}'", i, m.trim());
                        m.trim().to_uppercase()
                    },
                    Err(e) => {
                        tracing::error!("RPS: Player {} failed to send move: {:?}", i, e);
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
                        tracing::error!("RPS: Player {} sent invalid move: '{}'", i, move_str);
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
                tracing::debug!("RPS: Player {} move parsed as: {}", i, choice);
            }

            tracing::debug!("RPS: Both players submitted moves: {:?}", moves);

            // Determine winner: 0=rock, 1=paper, 2=scissors
            let winner = match (moves[0], moves[1]) {
                (a, b) if a == b => {
                    tracing::debug!("RPS: Round {} is a draw ({} vs {})", round, a, b);
                    None
                }, // Draw
                (0, 2) | (1, 0) | (2, 1) => {
                    tracing::debug!("RPS: Round {} won by player 0 ({} vs {})", round, moves[0], moves[1]);
                    Some(0)
                }, // Player 0 wins
                _ => {
                    tracing::debug!("RPS: Round {} won by player 1 ({} vs {})", round, moves[0], moves[1]);
                    Some(1)
                }, // Player 1 wins
            };

            // Update state
            {
                let mut state = self.state.lock().unwrap();
                if let Some(w) = winner {
                    state.scores[w] += 1;
                    tracing::debug!("RPS: Updated score for player {}, new scores: {:?}", w, state.scores);
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
            tracing::debug!("RPS: Sending round result: '{}'", result_msg);
            let result_0 = players[0].send_message(&result_msg).await;
            let result_1 = players[1].send_message(&result_msg).await;
            tracing::debug!("RPS: Round result send status - P0: {:?}, P1: {:?}", result_0, result_1);
        }

        // Mark game as finished
        let final_scores = {
            let mut state = self.state.lock().unwrap();
            state.game_finished = true;
            tracing::info!("RPS: Game finished, final scores: {:?}", state.scores);
            state.scores
        };

        // Send final results
        tracing::debug!("RPS: Sending final scores to players");
        let final_0 = players[0].send_message(&format!("SCORE {}", final_scores[0])).await;
        let final_1 = players[1].send_message(&format!("SCORE {}", final_scores[1])).await;
        tracing::debug!("RPS: Final score send status - P0: {:?}, P1: {:?}", final_0, final_1);

        // Send END message for graceful exit
        tracing::debug!("RPS: Sending END messages");
        let end_0 = players[0].send_message("END").await;
        let end_1 = players[1].send_message("END").await;
        tracing::debug!("RPS: END message send status - P0: {:?}, P1: {:?}", end_0, end_1);

        let final_msg = vec![GameResult::Accepted(final_scores[0]), GameResult::Accepted(final_scores[1])];
        tracing::info!("RPS: Game completed with results: {:?}", final_msg);

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
