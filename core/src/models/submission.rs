use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

use super::game::ProgrammingLanguage;

/// Canonical submission row. Judge writes a subset (status,
/// compiled_binary_path, error_message); API owns the rest. Fields
/// marked `#[serde(default)]` so judge's partial reads tolerate
/// missing values during transition.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Submission {
    #[serde(default)]
    pub id: Option<RecordId>,
    pub user_id: RecordId,
    pub tournament_id: RecordId,
    pub game_id: String,
    pub language: ProgrammingLanguage,
    pub code: String,
    pub status: SubmissionStatus,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub compiled_binary_path: Option<String>,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum SubmissionStatus {
    Pending,
    Compiling,
    Accepted,
    Failed,
}
