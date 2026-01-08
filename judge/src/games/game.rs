/// Interface for communication between server and client
/// Individual games implement their own logic
pub trait Game: Send + Sync + Clone + 'static {
    /// Create new game instance
    fn new() -> Self;

    /// Run the game with players and return results
    /// The game_context parameter allows games to write state changes to persistent history
    fn run(&self, players: Vec<Box<dyn crate::players::Player>>, timeout_ms: u64, game_context: crate::room::GameContext) -> impl std::future::Future<Output = Vec<GameResult>> + Send;

    /// Get the maximum number of players for this game
    fn max_players(&self) -> usize;

    /// Restore game state from event history
    /// Called when server restarts and needs to reconstruct ongoing games
    fn restore_from_events(&self, events: &[String]);

    /// Get reconnection state for a specific player
    /// Called when a player reconnects to restore their client state
    fn get_event_source(&self, player_id: &str) -> Vec<String>;
}

/// Result of a player's performance in the game
#[derive(Debug, Clone, serde::Serialize)]
pub enum GameResult {
    Accepted(i32),        // Score
    TimeLimitExceeded,
    WrongAnswer,
    RuntimeError,
}
