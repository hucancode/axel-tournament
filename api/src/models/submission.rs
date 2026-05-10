use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};
use validator::Validate;

use super::game::ProgrammingLanguage;
use super::{bare_key, opt_bare_key};

#[derive(Debug, Clone, Deserialize, SurrealValue)]
pub struct Submission {
    pub id: Option<RecordId>,
    pub user_id: RecordId,
    pub tournament_id: RecordId,
    pub game_id: String,
    pub language: ProgrammingLanguage,
    pub code: String,
    pub status: SubmissionStatus,
    pub error_message: Option<String>,
    pub compiled_binary_path: Option<String>,
    pub created_at: Datetime,
}

/// HTTP-safe view: hides `user_id`, `game_id`, `code`,
/// `error_message`, `compiled_binary_path` so source + private paths
/// don't leak.
#[derive(Debug, Clone, Serialize)]
pub struct SubmissionResponse {
    pub id: Option<String>,
    pub tournament_id: String,
    pub language: ProgrammingLanguage,
    pub status: SubmissionStatus,
    pub created_at: Datetime,
}

impl From<Submission> for SubmissionResponse {
    fn from(s: Submission) -> Self {
        Self {
            id: opt_bare_key(&s.id),
            tournament_id: bare_key(&s.tournament_id),
            language: s.language,
            status: s.status,
            created_at: s.created_at,
        }
    }
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubmissionRequest {
    pub tournament_id: String,
    pub language: String, // Will be validated and converted to ProgrammingLanguage
    #[validate(length(min = 1, max = 1048576, message = "Code must be 1 byte to 1MB"))]
    pub code: String,
}

