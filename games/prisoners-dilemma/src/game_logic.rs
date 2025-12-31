use anyhow::Result;
use game_framework::GameLogic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Move {
    Cooperate,
    Defect,
}

impl Move {
    pub fn to_str(&self) -> &str {
        match self {
            Move::Cooperate => "cooperate",
            Move::Defect => "defect",
        }
    }
}

fn calculate_scores(move1: Move, move2: Move) -> (i32, i32) {
    match (move1, move2) {
        (Move::Cooperate, Move::Cooperate) => (3, 3),
        (Move::Defect, Move::Defect) => (1, 1),
        (Move::Cooperate, Move::Defect) => (0, 5),
        (Move::Defect, Move::Cooperate) => (5, 0),
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

pub struct PrisonersDilemmaGame;

impl GameLogic for PrisonersDilemmaGame {
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
            "cooperate" | "c" => Ok(Move::Cooperate),
            "defect" | "d" => Ok(Move::Defect),
            _ => Err(anyhow::anyhow!("Invalid move: {}", input)),
        }
    }

    fn make_move(state: &mut Self::GameState, player_idx: usize, mv: &Self::Move) -> Result<()> {
        match player_idx {
            0 => state.last_move_player1 = Some(*mv),
            1 => state.last_move_player2 = Some(*mv),
            _ => return Err(anyhow::anyhow!("Invalid player index")),
        }

        // If both players have moved, score the round
        if let (Some(move1), Some(move2)) = (state.last_move_player1, state.last_move_player2) {
            let (points1, points2) = calculate_scores(move1, move2);
            state.score_player1 += points1;
            state.score_player2 += points2;
            state.round += 1;
        }

        Ok(())
    }

    fn is_game_over(state: &Self::GameState) -> bool {
        state.round >= 100
    }

    fn get_scores(state: &Self::GameState) -> Vec<i32> {
        vec![state.score_player1, state.score_player2]
    }

    fn encode_state_for_player(state: &Self::GameState, player_idx: usize) -> String {
        // Send opponent's last move (if any)
        let opponent_idx = 1 - player_idx;
        let opponent_move = if opponent_idx == 0 {
            state.last_move_player1
        } else {
            state.last_move_player2
        };

        if let Some(mv) = opponent_move {
            format!("OPP {}", mv.to_str())
        } else {
            "MOVE".to_string()
        }
    }

    fn get_round_result_message(
        state: &Self::GameState,
        player1_move: &Self::Move,
        player2_move: &Self::Move,
    ) -> Option<serde_json::Value> {
        let (points1, points2) = calculate_scores(*player1_move, *player2_move);

        Some(serde_json::json!({
            "your_move": player1_move.to_str(),
            "opponent_move": player2_move.to_str(),
            "your_points": points1,
            "opponent_points": points2,
            "your_score": state.score_player1,
            "opponent_score": state.score_player2,
        }))
    }
}
