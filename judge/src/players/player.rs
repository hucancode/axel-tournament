use anyhow::Result;
use async_trait::async_trait;
use surrealdb::sql::Thing;

/// Player trait - interface to communicate between server and client
/// Abstracts where player inputs come from and where outputs go to
#[async_trait]
pub trait Player: Send + Sync {
    /// Send message from GameLogic to player
    async fn send_message(&self, message: &str) -> Result<()>;

    /// Receive message from player using the configured timeout
    async fn receive_message(&self) -> Result<String>;

    /// Get player identifier for logging
    fn player_id(&self) -> &Thing;

    /// Check if player is still connected/alive
    async fn is_alive(&self) -> bool;

    /// Set the timeout for this player
    fn set_timeout(&mut self, timeout_ms: u64);
}
