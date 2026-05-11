use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

/// How a finished match contributes to the leaderboard.
/// `Elo` = 1v1 win/lose ELO update.
/// `Score` = accumulate per-player score (e.g. chips for poker, rounds won
/// for RPS/PD) into the participant's `elo` column. Score-based games also
/// use a different matchmaker (continuous score-pairing rather than bracket).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScoringKind {
    Elo,
    Score,
}

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
    pub scoring_kind: ScoringKind,
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
        scoring_kind: ScoringKind::Score,
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
        scoring_kind: ScoringKind::Score,
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
        scoring_kind: ScoringKind::Elo,
    },
    GameMetadata {
        id: "chess",
        name: "Chess",
        description: "Standard chess for 2 players",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 600000,
        human_timeout_ms: 1800000,
        cpu_limit: 2.0,
        bot_turn_timeout_ms: 60000,
        human_turn_timeout_ms: 120000,
        memory_limit_mb: 128,
        scoring_kind: ScoringKind::Elo,
    },
    GameMetadata {
        id: "xiangqi",
        name: "Xiangqi",
        description: "Chinese chess for 2 players",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 600000,
        human_timeout_ms: 1800000,
        cpu_limit: 2.0,
        bot_turn_timeout_ms: 60000,
        human_turn_timeout_ms: 120000,
        memory_limit_mb: 128,
        scoring_kind: ScoringKind::Elo,
    },
    GameMetadata {
        id: "poker",
        name: "Poker",
        description: "Heads-up no-limit Texas Hold'em",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 1,
        repetitions: 1,
        bot_timeout_ms: 600000,
        human_timeout_ms: 1800000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 15000,
        human_turn_timeout_ms: 60000,
        memory_limit_mb: 128,
        scoring_kind: ScoringKind::Score,
    },
    GameMetadata {
        id: "jar-of-greed",
        name: "Jar of Greed",
        description: "2-8 players secretly contribute to a shared jar that grows by a configurable factor before being split evenly",
        supported_languages: &[
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        rounds_per_match: 50,
        repetitions: 1,
        bot_timeout_ms: 5000,
        human_timeout_ms: 30000,
        cpu_limit: 1.0,
        bot_turn_timeout_ms: 2000,
        human_turn_timeout_ms: 15000,
        memory_limit_mb: 64,
        scoring_kind: ScoringKind::Score,
    },
];

// Helper function to find a game by ID
pub fn find_game_by_id(id: &str) -> Option<&'static GameMetadata> {
    GAMES.iter().find(|g| g.id == id)
}

// Type alias for response (same as metadata)
pub type GameResponse = GameMetadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
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
