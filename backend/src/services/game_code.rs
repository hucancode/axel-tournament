use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Game, ProgrammingLanguage},
};
use std::path::Path;
use surrealdb::sql::Datetime;
use tokio::fs;

pub async fn upload_game_code(
    db: &Database,
    game_id: &str,
    language: ProgrammingLanguage,
    code_content: String,
) -> ApiResult<String> {
    // Create game_code directory if it doesn't exist
    let game_code_dir = Path::new("game_code");
    if !game_code_dir.exists() {
        fs::create_dir_all(game_code_dir).await
            .map_err(|e| ApiError::Internal(format!("Failed to create game_code directory: {}", e)))?;
    }

    // Determine file extension based on language
    let extension = language.to_extension();

    // Save game code to filesystem
    let file_path = game_code_dir.join(format!("game_{}.{}", game_id, extension));
    fs::write(&file_path, &code_content).await
        .map_err(|e| ApiError::Internal(format!("Failed to write game code: {}", e)))?;

    // Update game record
    let game_id_clean = game_id.trim_start_matches("game:");
    let game: Option<Game> = db.select(("game", game_id_clean)).await?;
    let mut game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.game_code_path = Some(file_path.to_string_lossy().to_string());
    game.game_language = Some(language);
    game.updated_at = Datetime::default();

    let _: Option<Game> = db.update(("game", game_id_clean)).content(game).await?;

    Ok(file_path.to_string_lossy().to_string())
}

pub async fn get_game_code_path(db: &Database, game_id: &str) -> ApiResult<String> {
    let game_id_clean = game_id.trim_start_matches("game:");
    let game: Option<Game> = db.select(("game", game_id_clean)).await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.game_code_path
        .ok_or_else(|| ApiError::NotFound("Game code not found for this game".to_string()))
}
