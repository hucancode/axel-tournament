use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPolicy {
    pub id: Option<Thing>,
    pub tournament_id: Thing,
    pub rounds_per_match: u32,
    pub repetitions: u32,
    pub timeout_seconds: u32,
    pub cpu_limit: Option<String>,    // e.g., "1.0" for 1 CPU
    pub memory_limit: Option<String>, // e.g., "512m"
    pub scoring_weights: Option<serde_json::Value>, // JSON for custom weights
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMatchPolicyRequest {
    pub tournament_id: String,
    #[validate(range(min = 1, max = 100, message = "Rounds per match must be 1-100"))]
    pub rounds_per_match: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Repetitions must be 1-100"))]
    pub repetitions: Option<u32>,
    #[validate(range(min = 1, max = 3600, message = "Timeout must be 1-3600 seconds"))]
    pub timeout_seconds: Option<u32>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub scoring_weights: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMatchPolicyRequest {
    #[validate(range(min = 1, max = 100, message = "Rounds per match must be 1-100"))]
    pub rounds_per_match: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Repetitions must be 1-100"))]
    pub repetitions: Option<u32>,
    #[validate(range(min = 1, max = 3600, message = "Timeout must be 1-3600 seconds"))]
    pub timeout_seconds: Option<u32>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub scoring_weights: Option<serde_json::Value>,
}
