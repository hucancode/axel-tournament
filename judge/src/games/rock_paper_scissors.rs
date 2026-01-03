use crate::game_logic::{GameLogic, GameResult};
use crate::player::Player;

#[derive(Clone)]
pub struct RockPaperScissors;

impl GameLogic for RockPaperScissors {
    fn new() -> Self {
        RockPaperScissors
    }

    async fn run(&self, mut players: Vec<Box<dyn Player>>, timeout_ms: u64) -> Vec<GameResult> {
        if players.len() != 2 {
            return vec![GameResult::RuntimeError; players.len()];
        }

        // Set timeout on all players
        for player in &mut players {
            player.set_timeout(timeout_ms);
        }

        let mut scores = [0, 0];
        let rounds = 5;

        // Send start messages
        let _ = players[0].send_message("START 5").await;
        let _ = players[1].send_message("START 5").await;

        for round in 1..=rounds {
            let mut moves = Vec::new();

            // Get moves from both players simultaneously
            for i in 0..2 {
                let move_str = match players[i].receive_message().await {
                    Ok(m) => m.trim().to_uppercase(),
                    Err(_) => return if i == 0 { 
                        vec![GameResult::TimeLimitExceeded, GameResult::Accepted(scores[1])] 
                    } else { 
                        vec![GameResult::Accepted(scores[0]), GameResult::TimeLimitExceeded] 
                    }
                };

                let choice = match move_str.as_str() {
                    "ROCK" => 0,
                    "PAPER" => 1,
                    "SCISSORS" => 2,
                    _ => return if i == 0 { 
                        vec![GameResult::WrongAnswer, GameResult::Accepted(scores[1])] 
                    } else { 
                        vec![GameResult::Accepted(scores[0]), GameResult::WrongAnswer] 
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

            if let Some(w) = winner {
                scores[w] += 1;
            }

            // Send round results
            let result_msg = format!("ROUND {} SCORE {} {}", round, scores[0], scores[1]);
            let _ = players[0].send_message(&result_msg).await;
            let _ = players[1].send_message(&result_msg).await;
        }

        // Send final results
        let final_msg = if scores[0] > scores[1] {
            let _ = players[0].send_message("WIN").await;
            let _ = players[1].send_message("LOSE").await;
            vec![GameResult::Accepted(scores[0]), GameResult::Accepted(scores[1])]
        } else if scores[1] > scores[0] {
            let _ = players[0].send_message("LOSE").await;
            let _ = players[1].send_message("WIN").await;
            vec![GameResult::Accepted(scores[0]), GameResult::Accepted(scores[1])]
        } else {
            let _ = players[0].send_message("DRAW").await;
            let _ = players[1].send_message("DRAW").await;
            vec![GameResult::Accepted(scores[0]), GameResult::Accepted(scores[1])]
        };

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
}
