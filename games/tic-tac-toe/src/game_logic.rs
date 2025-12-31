use anyhow::Result;
use game_framework::GameLogic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Move(pub usize); // 0-8 board position

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: [char; 9],
    pub current_player: usize, // 0 or 1
    pub game_over: bool,
    pub winner: Option<usize>,
}

impl GameState {
    fn check_winner(&self) -> Option<char> {
        let lines = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8], // rows
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8], // columns
            [0, 4, 8],
            [2, 4, 6], // diagonals
        ];

        for line in lines {
            if self.board[line[0]] != ' '
                && self.board[line[0]] == self.board[line[1]]
                && self.board[line[1]] == self.board[line[2]]
            {
                return Some(self.board[line[0]]);
            }
        }
        None
    }

    fn is_draw(&self) -> bool {
        self.board.iter().all(|&cell| cell != ' ') && self.check_winner().is_none()
    }

    pub fn board_to_string(&self) -> String {
        self.board
            .iter()
            .map(|&c| if c == ' ' { '.' } else { c })
            .collect()
    }
}

pub struct TicTacToeGame;

impl GameLogic for TicTacToeGame {
    type Move = Move;
    type GameState = GameState;

    fn new_game() -> Self::GameState {
        GameState {
            board: [' '; 9],
            current_player: 0,
            game_over: false,
            winner: None,
        }
    }

    fn parse_move(input: &str) -> Result<Self::Move> {
        let position = input
            .trim()
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Invalid position: {}", input))?;

        if position >= 9 {
            return Err(anyhow::anyhow!("Position out of range: {}", position));
        }

        Ok(Move(position))
    }

    fn make_move(state: &mut Self::GameState, player_idx: usize, mv: &Self::Move) -> Result<()> {
        if state.game_over {
            return Err(anyhow::anyhow!("Game is over"));
        }

        if player_idx != state.current_player {
            return Err(anyhow::anyhow!("Not your turn"));
        }

        let position = mv.0;
        if state.board[position] != ' ' {
            return Err(anyhow::anyhow!("Position already taken"));
        }

        // Place mark (X for player 0, O for player 1)
        state.board[position] = if player_idx == 0 { 'X' } else { 'O' };

        // Check for winner
        if let Some(winner_char) = state.check_winner() {
            state.game_over = true;
            state.winner = Some(if winner_char == 'X' { 0 } else { 1 });
        } else if state.is_draw() {
            state.game_over = true;
            state.winner = None;
        } else {
            // Switch player
            state.current_player = 1 - state.current_player;
        }

        Ok(())
    }

    fn is_game_over(state: &Self::GameState) -> bool {
        state.game_over
    }

    fn get_scores(state: &Self::GameState) -> Vec<i32> {
        match state.winner {
            Some(0) => vec![3, 0], // Player 1 wins: 3 points
            Some(1) => vec![0, 3], // Player 2 wins: 3 points
            None if state.game_over => vec![1, 1], // Draw: 1 point each
            _ => vec![0, 0],
        }
    }

    fn encode_state_for_player(state: &Self::GameState, player_idx: usize) -> String {
        // Send player symbol first (X or O), then board state
        let symbol = if player_idx == 0 { 'X' } else { 'O' };
        format!("{}\n{}", symbol, state.board_to_string())
    }

    fn get_state_message(state: &Self::GameState) -> serde_json::Value {
        serde_json::json!({
            "type": "game_state",
            "board": state.board,
            "current_player": state.current_player,
            "game_over": state.game_over,
            "winner": state.winner,
        })
    }

    fn get_round_result_message(
        state: &Self::GameState,
        _player1_move: &Self::Move,
        _player2_move: &Self::Move,
    ) -> Option<serde_json::Value> {
        // For tic-tac-toe, just return updated board state
        Some(Self::get_state_message(state))
    }

    fn get_game_over_message(state: &Self::GameState) -> serde_json::Value {
        let scores = Self::get_scores(state);
        serde_json::json!({
            "type": "game_over",
            "board": state.board,
            "winner": state.winner,
            "scores": scores,
        })
    }
}
