use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{GameTemplate, ProgrammingLanguage},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_template(
    db: &Database,
    game_id: String,
    language: String,
    template_code: String,
) -> ApiResult<GameTemplate> {
    // Validate language
    let _lang = ProgrammingLanguage::from_str(&language)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid language: {}", language)))?;

    let game_id_thing = Thing::from(("game", game_id.trim_start_matches("game:")));

    let template = GameTemplate {
        id: None,
        game_id: game_id_thing,
        language,
        template_code,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };

    let created: Option<GameTemplate> = db.create("game_template").content(template).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create template".to_string()))
}

pub async fn get_template(db: &Database, game_id: &str, language: &str) -> ApiResult<GameTemplate> {
    let game_id_clean = game_id.trim_start_matches("game:");
    let language_owned = language.to_string();
    let mut result = db
        .query("SELECT * FROM game_template WHERE game_id = $game_id AND language = $language")
        .bind(("game_id", Thing::from(("game", game_id_clean))))
        .bind(("language", language_owned))
        .await?;

    let templates: Vec<GameTemplate> = result.take(0)?;
    templates.into_iter().next()
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))
}

pub async fn list_templates(db: &Database, game_id: &str) -> ApiResult<Vec<GameTemplate>> {
    let game_id_clean = game_id.trim_start_matches("game:");
    let mut result = db
        .query("SELECT * FROM game_template WHERE game_id = $game_id")
        .bind(("game_id", Thing::from(("game", game_id_clean))))
        .await?;

    let templates: Vec<GameTemplate> = result.take(0)?;
    Ok(templates)
}

pub async fn update_template(
    db: &Database,
    game_id: &str,
    language: &str,
    template_code: String,
) -> ApiResult<GameTemplate> {
    let existing = get_template(db, game_id, language).await?;

    let mut updated = existing;
    updated.template_code = template_code;
    updated.updated_at = Datetime::default();

    let template_id = updated.id.clone().unwrap();
    let result: Option<GameTemplate> = db
        .update((template_id.tb.as_str(), template_id.id.to_string()))
        .content(updated)
        .await?;

    result.ok_or_else(|| ApiError::Internal("Failed to update template".to_string()))
}
