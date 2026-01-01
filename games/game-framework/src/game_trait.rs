use anyhow::Result;

/// Core trait that all games must implement
/// Games only need to focus on their specific logic
pub trait GameLogic: Send + Sync + 'static {
    /// The move type for this game (e.g., Rock/Paper/Scissors, or board position)
    type Move: Clone + Send + Sync;

    /// The game state (e.g., scores, board, current player)
    type GameState: Clone + Send + Sync + std::fmt::Debug;

    /// Create a new game state
    fn new_game() -> Self::GameState;

    /// Parse a move from player input string
    /// Returns error if invalid format
    fn parse_move(input: &str) -> Result<Self::Move>;

    /// Apply a move to the game state
    /// player_idx: 0 for player 1, 1 for player 2
    /// Returns error if move is invalid
    fn make_move(state: &mut Self::GameState, player_idx: usize, mv: &Self::Move) -> Result<()>;

    /// Check if the game is over
    fn is_game_over(state: &Self::GameState) -> bool;

    /// Get the scores for both players [player1_score, player2_score]
    fn get_scores(state: &Self::GameState) -> Vec<i32>;

    /// Encode the current game state as a string to send to a player
    /// player_idx: which player's perspective (0 or 1)
    /// This is sent to bots via stdin and to humans via WebSocket
    fn encode_state_for_player(state: &Self::GameState, player_idx: usize) -> String;

    /// Optional: Get WebSocket message for game state update
    /// Default implementation sends state as plain text
    fn get_state_message(state: &Self::GameState) -> serde_json::Value {
        serde_json::json!({
            "type": "game_state",
            "state": format!("{:?}", state)
        })
    }

    /// Optional: Get round result message for WebSocket
    fn get_round_result_message(
        state: &Self::GameState,
        _player1_move: &Self::Move,
        _player2_move: &Self::Move,
    ) -> Option<serde_json::Value> {
        let scores = Self::get_scores(state);
        Some(serde_json::json!({
            "type": "round_result",
            "score_player1": scores[0],
            "score_player2": scores[1]
        }))
    }

    /// Optional: Get game over message for WebSocket
    fn get_game_over_message(state: &Self::GameState) -> serde_json::Value {
        let scores = Self::get_scores(state);
        let winner = if scores[0] > scores[1] {
            Some(0)
        } else if scores[1] > scores[0] {
            Some(1)
        } else {
            None
        };

        serde_json::json!({
            "type": "game_over",
            "final_scores": scores,
            "winner": winner
        })
    }
}

/// Configuration for automated matches
#[derive(Clone, Debug)]
pub struct GameConfig {
    pub num_rounds: u32,
    pub turn_timeout_ms: u64,
    pub memory_limit_mb: i64,
    pub cpu_limit: f64,
}
