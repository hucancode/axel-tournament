use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTemplate {
    pub id: Option<Thing>,
    pub game_id: Thing,
    pub language: String, // ProgrammingLanguage as string
    pub template_code: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGameTemplateRequest {
    pub game_id: String,
    pub language: String,
    #[validate(length(
        min = 1,
        max = 1048576,
        message = "Template code must be 1-1048576 characters"
    ))]
    pub template_code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGameTemplateRequest {
    #[validate(length(
        min = 1,
        max = 1048576,
        message = "Template code must be 1-1048576 characters"
    ))]
    pub template_code: String,
}
