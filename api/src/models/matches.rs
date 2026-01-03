use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: Option<Thing>,
    pub tournament_id: Option<Thing>, // Optional for standalone interactive matches
    pub game_id: String, // Changed from Thing - games are now hardcoded
    pub status: MatchStatus,
    pub participants: Vec<MatchParticipant>,
    pub metadata: Option<serde_json::Value>, // For game-specific replay data or logs
    pub room_id: Option<Thing>, // For interactive matches
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub started_at: Option<Datetime>,
    pub completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchResponse {
    pub id: String,
    pub tournament_id: Option<String>,
    pub game_id: String,
    pub status: MatchStatus,
    pub participants: Vec<MatchParticipantResponse>,
    pub metadata: Option<serde_json::Value>,
    pub room_id: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub started_at: Option<Datetime>,
    pub completed_at: Option<Datetime>,
}

impl From<Match> for MatchResponse {
    fn from(match_data: Match) -> Self {
        Self {
            id: match_data.id.map(|t| t.to_string()).unwrap_or_default(),
            tournament_id: match_data.tournament_id.map(|t| t.to_string()),
            game_id: match_data.game_id, // Already a String
            status: match_data.status,
            participants: match_data.participants.into_iter().map(Into::into).collect(),
            metadata: match_data.metadata,
            room_id: match_data.room_id.map(|t| t.to_string()),
            created_at: match_data.created_at,
            updated_at: match_data.updated_at,
            started_at: match_data.started_at,
            completed_at: match_data.completed_at,
        }
    }
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
    pub user_id: Thing,
    pub submission_id: Option<Thing>, // For automated matches
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchParticipantResponse {
    pub user_id: String,
    pub submission_id: Option<String>,
    pub score: Option<f64>,
}

impl From<MatchParticipant> for MatchParticipantResponse {
    fn from(participant: MatchParticipant) -> Self {
        Self {
            user_id: participant.user_id.to_string(),
            submission_id: participant.submission_id.map(|t| t.to_string()),
            score: participant.score,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMatchRequest {
    pub tournament_id: String,
    pub game_id: String,

    #[validate(length(min = 2, message = "Match must have at least 2 participants"))]
    pub participant_submission_ids: Vec<String>,
}
