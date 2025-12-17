use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{GameTemplate, ProgrammingLanguage},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_template(
    db: &Database,
    game_id: Thing,
    language: String,
    template_code: String,
) -> ApiResult<GameTemplate> {
    // Validate language
    let _lang = ProgrammingLanguage::from_str(&language)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid language: {}", language)))?;

    let template = GameTemplate {
        id: None,
        game_id,
        language,
        template_code,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };

    let created: Option<GameTemplate> = db.create("game_template").content(template).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create template".to_string()))
}

pub async fn get_template(db: &Database, game_id: Thing, language: &str) -> ApiResult<GameTemplate> {
    let language_owned = language.to_string();
    let mut result = db
        .query("SELECT * FROM game_template WHERE game_id = $game_id AND language = $language")
        .bind(("game_id", game_id))
        .bind(("language", language_owned))
        .await?;

    let templates: Vec<GameTemplate> = result.take(0)?;
    templates.into_iter().next()
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))
}

pub async fn list_templates(db: &Database, game_id: Thing) -> ApiResult<Vec<GameTemplate>> {
    let mut result = db
        .query("SELECT * FROM game_template WHERE game_id = $game_id")
        .bind(("game_id", game_id))
        .await?;

    let templates: Vec<GameTemplate> = result.take(0)?;
    Ok(templates)
}

pub async fn update_template(
    db: &Database,
    game_id: Thing,
    language: &str,
    template_code: String,
) -> ApiResult<GameTemplate> {
    let existing = get_template(db, game_id.clone(), language).await?;

    let mut updated = existing;
    updated.template_code = template_code;
    updated.updated_at = Datetime::default();

    let template_id = updated.id.clone().unwrap();
    let key = (template_id.tb.as_str(), template_id.id.to_string());
    let result: Option<GameTemplate> = db.update(key).content(updated).await?;

    result.ok_or_else(|| ApiError::Internal("Failed to update template".to_string()))
}
