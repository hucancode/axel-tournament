// Core modules for game infrastructure
pub mod automated_executor;
pub mod db_client;
pub mod docker_player;
pub mod game_trait;
pub mod match_watcher;
pub mod server;
pub mod websocket_room;

// Legacy modules (keeping for backward compatibility)
pub mod automated;
pub mod interactive;

// Re-exports for convenience
pub use automated_executor::execute_automated_match;
pub use db_client::{DbClient, MatchParticipant, MatchRow, ParticipantResult, Submission};
pub use docker_player::DockerPlayer;
pub use game_trait::{GameConfig, GameLogic};
pub use match_watcher::{run_match_watcher, MatchWatcherConfig};
pub use server::run_http_server;
pub use websocket_room::{websocket_handler, AppState, GameMessage};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GameType {
    Automated,
    Interactive,
}

impl Default for GameType {
    fn default() -> Self {
        GameType::Automated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProgrammingLanguage {
    Rust,
    Go,
    C,
}

impl ProgrammingLanguage {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "c" => Some(Self::C),
            _ => None,
        }
    }

    pub fn to_extension(&self) -> &str {
        match self {
            Self::Rust => "rs",
            Self::Go => "go",
            Self::C => "c",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub game_type: GameType,
    pub supported_languages: Vec<ProgrammingLanguage>,
    pub server_port: u16,
    pub rounds_per_match: u32,
    pub repetitions: u32,
    pub timeout_ms: u32,
    pub cpu_limit: f64,
    pub turn_timeout_ms: u64,
    pub memory_limit_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchExecutionResult {
    pub participant_results: Vec<ParticipantResult>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Player process error: {0}")]
    PlayerProcessError(String),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Timeout error")]
    TimeoutError,

    #[error("Invalid move: {0}")]
    InvalidMoveError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type GameResult<T> = Result<T, GameError>;
