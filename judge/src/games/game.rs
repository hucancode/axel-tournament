/// Interface for communication between server and client
/// Individual games implement their own logic
pub trait Game: Send + Sync + Clone + 'static {
    /// Create new game instance
    fn new() -> Self;

    /// Run the game with players and return results
    fn run(&self, players: Vec<Box<dyn crate::players::Player>>, timeout_ms: u64) -> impl std::future::Future<Output = Vec<GameResult>> + Send;

    /// Get the game's unique identifier
    #[allow(dead_code)]
    fn game_id(&self) -> &'static str;

    /// Get the maximum number of players for this game
    fn max_players(&self) -> usize;

    /// Get state messages to replay for a reconnecting player
    fn get_reconnect_state(&self, player_id: &str) -> Vec<String>;
}

/// Result of a player's performance in the game
#[derive(Debug, Clone, serde::Serialize)]
pub enum GameResult {
    Accepted(i32),        // Score
    TimeLimitExceeded,
    WrongAnswer,
    RuntimeError,
}
