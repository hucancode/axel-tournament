use crate::game_logic::{GameLogic, GameResult};
use crate::player::Player;

#[derive(Clone)]
pub struct PrisonersDilemma;

impl GameLogic for PrisonersDilemma {
    fn new() -> Self {
        PrisonersDilemma
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
        let rounds = 10;

        // Send start messages
        let _ = players[0].send_message("START 10").await;
        let _ = players[1].send_message("START 10").await;

        for _round in 1..=rounds {
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
                    "C" | "COOPERATE" => 0, // Cooperate
                    "D" | "DEFECT" => 1,    // Defect
                    _ => return if i == 0 { 
                        vec![GameResult::WrongAnswer, GameResult::Accepted(scores[1])] 
                    } else { 
                        vec![GameResult::Accepted(scores[0]), GameResult::WrongAnswer] 
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

            scores[0] += score0;
            scores[1] += score1;

            // Send round results
            let choice_str = |c| if c == 0 { "C" } else { "D" };
            let result_msg = format!("RESULT {} {} {} {}", 
                choice_str(moves[1]), choice_str(moves[0]), scores[0], scores[1]);
            let _ = players[0].send_message(&result_msg).await;
            
            let result_msg = format!("RESULT {} {} {} {}", 
                choice_str(moves[0]), choice_str(moves[1]), scores[1], scores[0]);
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

        final_msg
    }

    fn game_id(&self) -> &'static str {
        "prisoners-dilemma"
    }

    fn max_players(&self) -> usize {
        2
    }
}
