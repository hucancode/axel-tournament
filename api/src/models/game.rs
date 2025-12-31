use serde::{Deserialize, Serialize};

// Hardcoded game metadata (only serialized, never deserialized)
#[derive(Debug, Clone, Serialize)]
pub struct GameMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub game_type: GameType,
    pub supported_languages: &'static [ProgrammingLanguage],
    pub server_port: u16,
    pub rounds_per_match: u32,
    pub repetitions: u32,
    pub timeout_ms: u32,
    pub cpu_limit: f64,
    pub turn_timeout_ms: u64,
    pub memory_limit_mb: u64,
}

// Static game registry
pub static GAMES: &[GameMetadata] = &[
    GameMetadata {
        id: "rock-paper-scissors",
        name: "Rock Paper Scissors",
        description: "Classic rock-paper-scissors game for 2 players",
        game_type: GameType::Automated,
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        server_port: 8082,
        rounds_per_match: 100,
        repetitions: 1,
        timeout_ms: 5000,
        cpu_limit: 1.0,
        turn_timeout_ms: 2000,
        memory_limit_mb: 64,
    },
    GameMetadata {
        id: "prisoners-dilemma",
        name: "Prisoner's Dilemma",
        description: "Classic game theory prisoner's dilemma",
        game_type: GameType::Automated,
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        server_port: 8083,
        rounds_per_match: 100,
        repetitions: 1,
        timeout_ms: 5000,
        cpu_limit: 1.0,
        turn_timeout_ms: 2000,
        memory_limit_mb: 64,
    },
    GameMetadata {
        id: "tic-tac-toe",
        name: "Tic Tac Toe",
        description: "Interactive tic-tac-toe game for 2 players",
        game_type: GameType::Interactive,
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        server_port: 8084,
        rounds_per_match: 1,
        repetitions: 1,
        timeout_ms: 60000, // 1 minute for interactive game
        cpu_limit: 1.0,
        turn_timeout_ms: 30000, // 30 seconds per move
        memory_limit_mb: 64,
    },
];

// Helper function to find a game by ID
pub fn find_game_by_id(id: &str) -> Option<&'static GameMetadata> {
    GAMES.iter().find(|g| g.id == id)
}

// Type alias for response (same as metadata)
pub type GameResponse = GameMetadata;

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

