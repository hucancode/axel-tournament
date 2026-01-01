use anyhow::Result;
use game_framework::GameLogic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    pub fn beats(&self, other: &Move) -> bool {
        matches!(
            (self, other),
            (Move::Rock, Move::Scissors) | (Move::Paper, Move::Rock) | (Move::Scissors, Move::Paper)
        )
    }

    pub fn to_str(&self) -> &str {
        match self {
            Move::Rock => "rock",
            Move::Paper => "paper",
            Move::Scissors => "scissors",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub round: u32,
    pub score_player1: i32,
    pub score_player2: i32,
    pub last_move_player1: Option<Move>,
    pub last_move_player2: Option<Move>,
}

pub struct RockPaperScissorsGame;

impl GameLogic for RockPaperScissorsGame {
    type Move = Move;
    type GameState = GameState;

    fn new_game() -> Self::GameState {
        GameState {
            round: 0,
            score_player1: 0,
            score_player2: 0,
            last_move_player1: None,
            last_move_player2: None,
        }
    }

    fn parse_move(input: &str) -> Result<Self::Move> {
        match input.to_lowercase().trim() {
            "rock" | "r" => Ok(Move::Rock),
            "paper" | "p" => Ok(Move::Paper),
            "scissors" | "s" => Ok(Move::Scissors),
            _ => Err(anyhow::anyhow!("Invalid move: {}", input)),
        }
    }

    fn make_move(state: &mut Self::GameState, player_idx: usize, mv: &Self::Move) -> Result<()> {
        // Store moves
        match player_idx {
            0 => state.last_move_player1 = Some(*mv),
            1 => state.last_move_player2 = Some(*mv),
            _ => return Err(anyhow::anyhow!("Invalid player index")),
        }

        // If both players have moved, score the round
        if let (Some(move1), Some(move2)) = (state.last_move_player1, state.last_move_player2) {
            if move1.beats(&move2) {
                state.score_player1 += 1;
            } else if move2.beats(&move1) {
                state.score_player2 += 1;
            }
            // Draw: no score change

            state.round += 1;

            // Clear moves for next round
            state.last_move_player1 = None;
            state.last_move_player2 = None;
        }

        Ok(())
    }

    fn is_game_over(state: &Self::GameState) -> bool {
        state.round >= 100
    }

    fn get_scores(state: &Self::GameState) -> Vec<i32> {
        vec![state.score_player1, state.score_player2]
    }

    fn encode_state_for_player(state: &Self::GameState, _player_idx: usize) -> String {
        // For automated play: just send "READY" to request a move
        // Players don't need game state in RPS (no history dependency)
        "READY".to_string()
    }

    fn get_round_result_message(
        state: &Self::GameState,
        player1_move: &Self::Move,
        player2_move: &Self::Move,
    ) -> Option<serde_json::Value> {
        let result_p1 = if player1_move.beats(player2_move) {
            "win"
        } else if player2_move.beats(player1_move) {
            "lose"
        } else {
            "draw"
        };

        let result_p2 = if player2_move.beats(player1_move) {
            "win"
        } else if player1_move.beats(player2_move) {
            "lose"
        } else {
            "draw"
        };

        Some(serde_json::json!({
            "your_move": player1_move.to_str(),
            "opponent_move": player2_move.to_str(),
            "result": result_p1,
            "your_score": state.score_player1,
            "opponent_score": state.score_player2,
        }))
    }
}
