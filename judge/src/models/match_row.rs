// DB row shapes shared across services that read/write the `match`
// and `submission` collections. Lifted out of individual workers to
// avoid 3-way drift.

use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Match {
    pub id: RecordId,
    pub tournament_id: RecordId,
    pub game_id: String,
    pub status: String,
    pub participants: Vec<MatchParticipant>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub room_id: Option<RecordId>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    #[serde(default)]
    pub started_at: Option<Datetime>,
    #[serde(default)]
    pub completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MatchParticipant {
    #[serde(default)]
    pub user_id: Option<RecordId>,
    pub submission_id: RecordId,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Submission {
    #[serde(default)]
    pub id: Option<RecordId>,
    #[serde(default)]
    pub compiled_binary_path: Option<String>,
    pub language: String,
    pub code: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

/// Subset used by the compile worker — id is required (we only read
/// rows that have one) and we don't care about compile state.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PendingSubmission {
    pub id: RecordId,
    pub language: String,
    pub code: String,
}
