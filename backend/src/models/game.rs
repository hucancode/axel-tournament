use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
    pub rules: serde_json::Value, // JSON object with game-specific rules
    pub supported_languages: Vec<ProgrammingLanguage>,
    pub is_active: bool,
    pub created_at: Datetime,
    pub updated_at: Datetime,
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGameRequest {
    #[validate(length(min = 1, max = 100, message = "Game name must be 1-100 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 1000, message = "Description must be 1-1000 characters"))]
    pub description: String,
    pub rules: serde_json::Value,
    pub supported_languages: Vec<ProgrammingLanguage>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGameRequest {
    #[validate(length(min = 1, max = 100, message = "Game name must be 1-100 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 1000, message = "Description must be 1-1000 characters"))]
    pub description: Option<String>,
    pub rules: Option<serde_json::Value>,
    pub supported_languages: Option<Vec<ProgrammingLanguage>>,
    pub is_active: Option<bool>,
}
