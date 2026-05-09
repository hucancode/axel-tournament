use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue, ToSql};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct GameTemplate {
    pub id: Option<RecordId>,
    pub game_id: RecordId,
    pub language: String, // ProgrammingLanguage as string
    pub template_code: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameTemplateResponse {
    pub id: String,
    pub game_id: String,
    pub language: String,
    pub template_code: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<GameTemplate> for GameTemplateResponse {
    fn from(template: GameTemplate) -> Self {
        Self {
            id: template.id.map(|t| t.to_sql()).unwrap_or_default(),
            game_id: template.game_id.to_sql(),
            language: template.language,
            template_code: template.template_code,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
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
