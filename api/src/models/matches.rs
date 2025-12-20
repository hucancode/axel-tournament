use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: Option<Thing>,
    pub tournament_id: Option<Thing>,
    pub game_id: Thing,
    pub status: MatchStatus,
    pub participants: Vec<MatchParticipant>,
    pub metadata: Option<serde_json::Value>, // For game-specific replay data or logs
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub started_at: Option<Datetime>,
    pub completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchParticipant {
    pub submission_id: Thing,
    pub score: Option<f64>,
    pub metadata: Option<serde_json::Value>, // Player specific stats
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMatchRequest {
    pub tournament_id: Option<String>,
    pub game_id: String,

    #[validate(length(min = 2, message = "Match must have at least 2 participants"))]
    pub participant_submission_ids: Vec<String>,
}
