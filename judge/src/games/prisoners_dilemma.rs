use crate::games::{Game, GameResult};
use crate::players::Player;
use rand::Rng;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PrisonersDilemma {
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

impl Game for PrisonersDilemma {
    fn new() -> Self {
        PrisonersDilemma {
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
        const ROUNDS: u32 = 10;
        const ROUND_VAR: u32 = 3;

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
                    "C" | "COOPERATE" => 0, // Cooperate
                    "D" | "DEFECT" => 1,    // Defect
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

            // Calculate scores based on prisoner's dilemma payoff matrix
            // (C,C) = (3,3), (C,D) = (0,5), (D,C) = (5,0), (D,D) = (1,1)
            let (score0, score1) = match (moves[0], moves[1]) {
                (0, 0) => (3, 3), // Both cooperate
                (0, 1) => (0, 5), // P0 cooperates, P1 defects
                (1, 0) => (5, 0), // P0 defects, P1 cooperates
                (1, 1) => (1, 1), // Both defect
                _ => unreachable!(),
            };

            // Update state
            {
                let mut state = self.state.lock().unwrap();
                state.scores[0] += score0;
                state.scores[1] += score1;

                let _current_scores = state.scores;

                // Record round result (just the moves)
                state.round_history.push([moves[0], moves[1]]);
            }

            let scores = {
                let state = self.state.lock().unwrap();
                state.scores
            };

            // Send round results
            let choice_str = |c| if c == 0 { "C" } else { "D" };
            let result_msg = format!("RESULT {} {} {} {}",
                choice_str(moves[1]), choice_str(moves[0]), scores[1], scores[0]);
            let _ = players[0].send_message(&result_msg).await;

            let result_msg = format!("RESULT {} {} {} {}",
                choice_str(moves[0]), choice_str(moves[1]), scores[0], scores[1]);
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

        final_msg
    }

    fn game_id(&self) -> &'static str {
        "prisoners-dilemma"
    }

    fn max_players(&self) -> usize {
        2
    }

    fn get_reconnect_state(&self, player_id: &str) -> Vec<String> {
        let state = self.state.lock().unwrap();

        if !state.game_started {
            return vec![];
        }

        let mut messages = vec![];
        messages.push("START".to_string());

        // Determine player number from stored player_ids
        let player_num = state.player_ids.iter().position(|id| id == player_id).unwrap_or(0);

        // Replay all completed rounds with RESULT messages (same format as live gameplay)
        let mut scores = [0, 0];
        let choice_str = |c| if c == 0 { "C" } else { "D" };

        for moves in state.round_history.iter() {
            // Calculate scores for this round
            let (score0, score1) = match (moves[0], moves[1]) {
                (0, 0) => (3, 3), // Both cooperate
                (0, 1) => (0, 5), // P0 cooperates, P1 defects
                (1, 0) => (5, 0), // P0 defects, P1 cooperates
                (1, 1) => (1, 1), // Both defect
                _ => unreachable!(),
            };

            scores[0] += score0;
            scores[1] += score1;

            // Send in player's perspective (opponent_move, your_move, opponent_score, your_score)
            if player_num == 0 {
                messages.push(format!("RESULT {} {} {} {}",
                    choice_str(moves[1]), choice_str(moves[0]), scores[1], scores[0]));
            } else {
                messages.push(format!("RESULT {} {} {} {}",
                    choice_str(moves[0]), choice_str(moves[1]), scores[0], scores[1]));
            }
        }

        // If game is finished, send final score and END
        if state.game_finished {
            messages.push(format!("SCORE {}", state.scores[player_num]));
            messages.push("END".to_string());
        }

        messages
    }
}
