use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
    pub game_type: GameType,
    pub supported_languages: Vec<ProgrammingLanguage>,
    pub is_active: bool,
    pub owner_id: Thing, // reference to user who created the game
    pub game_code: String, // game orchestration code content
    pub game_language: ProgrammingLanguage, // language of game code
    pub frontend_code: Option<String>, // HTML/CSS/JS for interactive games
    pub rounds_per_match: u32, // match policy
    pub repetitions: u32,
    pub timeout_ms: u32,
    pub cpu_limit: f64,
    pub turn_timeout_ms: u64, // per-turn timeout forwarded to game code
    pub memory_limit_mb: u64, // container memory cap for player processes
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub game_type: GameType,
    pub supported_languages: Vec<ProgrammingLanguage>,
    pub is_active: bool,
    pub owner_id: String,
    pub game_code: String,
    pub game_language: ProgrammingLanguage,
    pub frontend_code: Option<String>,
    pub rounds_per_match: u32,
    pub repetitions: u32,
    pub timeout_ms: u32,
    pub cpu_limit: f64,
    pub turn_timeout_ms: u64,
    pub memory_limit_mb: u64,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<Game> for GameResponse {
    fn from(game: Game) -> Self {
        Self {
            id: game.id.map(|t| t.to_string()).unwrap_or_default(),
            name: game.name,
            description: game.description,
            game_type: game.game_type,
            supported_languages: game.supported_languages,
            is_active: game.is_active,
            owner_id: game.owner_id.to_string(),
            game_code: game.game_code,
            game_language: game.game_language,
            frontend_code: game.frontend_code,
            rounds_per_match: game.rounds_per_match,
            repetitions: game.repetitions,
            timeout_ms: game.timeout_ms,
            cpu_limit: game.cpu_limit,
            turn_timeout_ms: game.turn_timeout_ms,
            memory_limit_mb: game.memory_limit_mb,
            created_at: game.created_at,
            updated_at: game.updated_at,
        }
    }
}

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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGameRequest {
    #[validate(length(min = 1, max = 100, message = "Game name must be 1-100 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 1000, message = "Description must be 1-1000 characters"))]
    pub description: String,
    pub game_type: GameType,
    pub supported_languages: Vec<ProgrammingLanguage>,
    #[validate(length(min = 1, max = 1048576, message = "Game code must be 1 byte to 1MB"))]
    pub game_code: String,
    pub game_language: ProgrammingLanguage,
    #[validate(length(max = 2097152, message = "Frontend code must be less than 2MB"))]
    pub frontend_code: Option<String>,
    #[validate(range(min = 1, max = 100, message = "Rounds per match must be 1-100"))]
    pub rounds_per_match: u32,
    #[validate(range(min = 1, max = 100, message = "Repetitions must be 1-100"))]
    pub repetitions: u32,
    #[validate(range(min = 100, max = 5000, message = "Timeout must be 100-5000ms (0.1s-5s)"))]
    pub timeout_ms: u32,
    #[validate(range(min = 0.1, max = 64.0, message = "CPU limit must be 0.1-64.0 cores"))]
    pub cpu_limit: f64,
    #[validate(range(min = 1, max = 2000, message = "Turn timeout must be 1-2000ms"))]
    pub turn_timeout_ms: u64,
    #[validate(range(min = 1, max = 8192, message = "Memory limit must be 2-8192 MB"))]
    pub memory_limit_mb: u64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGameRequest {
    #[validate(length(min = 1, max = 100, message = "Game name must be 1-100 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 1000, message = "Description must be 1-1000 characters"))]
    pub description: Option<String>,
    pub game_type: Option<GameType>,
    pub supported_languages: Option<Vec<ProgrammingLanguage>>,
    pub is_active: Option<bool>,
    #[serde(default)]
    #[validate(length(min = 1, max = 1048576, message = "Game code must be 1 byte to 1MB"))]
    pub game_code: Option<String>,
    pub game_language: Option<ProgrammingLanguage>,
    #[serde(default)]
    #[validate(length(max = 2097152, message = "Frontend code must be less than 2MB"))]
    pub frontend_code: Option<String>,
    #[validate(range(min = 1, max = 100, message = "Rounds per match must be 1-100"))]
    pub rounds_per_match: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Repetitions must be 1-100"))]
    pub repetitions: Option<u32>,
    #[validate(range(min = 100, max = 5000, message = "Timeout must be 100-5000ms (0.1s-5s)"))]
    pub timeout_ms: Option<u32>,
    #[validate(range(min = 0.1, max = 64.0, message = "CPU limit must be 0.1-64.0 cores"))]
    pub cpu_limit: Option<f64>,
    #[serde(default)]
    #[validate(range(min = 1, max = 2000, message = "Turn timeout must be 1-2000ms"))]
    pub turn_timeout_ms: Option<u64>,
    #[serde(default)]
    #[validate(range(min = 1, max = 8192, message = "Memory limit must be 1-8192 MB"))]
    pub memory_limit_mb: Option<u64>,
}
