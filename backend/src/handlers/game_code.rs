use crate::{
    error::{ApiError, ApiResult},
    models::{user::Claims, Game, ProgrammingLanguage, UploadGameCodeRequest, UserRole},
    services,
    AppState,
};
use axum::{extract::{Path, State}, Extension, Json};
use surrealdb::sql::Thing;
use validator::Validate;

pub async fn upload_game_code(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<String>,
    Json(payload): Json<UploadGameCodeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    payload.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Parse and validate language
    let language = ProgrammingLanguage::from_str(&payload.language)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid language: {}", payload.language)))?;

    // Get the game to check ownership and supported languages
    let game_id_thing: Thing = game_id
        .parse()
        .map_err(|_| ApiError::BadRequest("Invalid game id".to_string()))?;
    let game: Option<Game> = state
        .db
        .select((game_id_thing.tb.as_str(), game_id_thing.id.to_string()))
        .await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    // Verify ownership (owner or admin)
    let user_id = claims.sub.to_string();
    let is_owner = game
        .owner_id
        .as_ref()
        .map(|owner| owner.to_string() == user_id)
        .unwrap_or(false);

    if !is_owner && claims.role != UserRole::Admin {
        return Err(ApiError::Forbidden(
            "You don't have permission to upload game code for this game".to_string(),
        ));
    }

    // Verify language is in supported_languages
    if !game.supported_languages.contains(&language) {
        return Err(ApiError::BadRequest(format!(
            "Language {:?} is not supported by this game. Supported languages: {:?}",
            language, game.supported_languages
        )));
    }

    // Upload the game code
    let file_path = services::game_code::upload_game_code(
        &state.db,
        game_id_thing,
        language,
        payload.code_content,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "message": "Game code uploaded successfully",
        "file_path": file_path
    })))
}
