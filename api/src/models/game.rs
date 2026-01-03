use serde::{Deserialize, Serialize};

// Hardcoded game metadata (only serialized, never deserialized)
// All games support both automated (bot) and interactive (human) modes
#[derive(Debug, Clone, Serialize)]
pub struct GameMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub supported_languages: &'static [ProgrammingLanguage],
    pub rounds_per_match: u32,
    pub repetitions: u32,
    pub bot_timeout_ms: u32,
    pub human_timeout_ms: u32,
    pub cpu_limit: f64,
    pub bot_turn_timeout_ms: u64,
    pub human_turn_timeout_ms: u64,
    pub memory_limit_mb: u64,
}

// Static game registry
pub static GAMES: &[GameMetadata] = &[
    GameMetadata {
        id: "rock-paper-scissors",
        name: "Rock Paper Scissors",
        description: "Classic rock-paper-scissors game for 2 players",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 100,
        repetitions: 1,
        bot_timeout_ms: 5000,
        human_timeout_ms: 30000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 2000,
        human_turn_timeout_ms: 10000,
        memory_limit_mb: 64,
    },
    GameMetadata {
        id: "prisoners-dilemma",
        name: "Prisoner's Dilemma",
        description: "Classic game theory prisoner's dilemma",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 100,
        repetitions: 1,
        bot_timeout_ms: 5000,
        human_timeout_ms: 30000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 2000,
        human_turn_timeout_ms: 10000,
        memory_limit_mb: 64,
    },
    GameMetadata {
        id: "tic-tac-toe",
        name: "Tic Tac Toe",
        description: "Classic tic-tac-toe game for 2 players",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 60000,
        human_timeout_ms: 120000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 30000,
        human_turn_timeout_ms: 60000,
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
