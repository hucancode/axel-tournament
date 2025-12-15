use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

use super::game::ProgrammingLanguage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: Option<Thing>,
    pub user_id: Thing,
    pub tournament_id: Thing,
    pub game_id: Thing,
    pub language: ProgrammingLanguage,
    pub code: String,
    pub file_path: String, // Stored file path
    pub status: SubmissionStatus,
    pub error_message: Option<String>,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
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

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: String,
    pub tournament_id: String,
    pub language: ProgrammingLanguage,
    pub status: SubmissionStatus,
    pub created_at: Datetime,
}
