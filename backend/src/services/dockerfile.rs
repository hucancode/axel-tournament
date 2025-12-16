use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::Game,
};
use std::path::Path;
use surrealdb::sql::Datetime;
use tokio::fs;

pub async fn upload_dockerfile(
    db: &Database,
    game_id: &str,
    dockerfile_content: String,
) -> ApiResult<String> {
    // Create dockerfiles directory if it doesn't exist
    let dockerfile_dir = Path::new("dockerfiles");
    if !dockerfile_dir.exists() {
        fs::create_dir_all(dockerfile_dir).await
            .map_err(|e| ApiError::Internal(format!("Failed to create dockerfiles directory: {}", e)))?;
    }

    // Save Dockerfile to filesystem
    let file_path = dockerfile_dir.join(format!("game_{}.dockerfile", game_id));
    fs::write(&file_path, &dockerfile_content).await
        .map_err(|e| ApiError::Internal(format!("Failed to write Dockerfile: {}", e)))?;

    // Update game record
    let game_id_clean = game_id.trim_start_matches("game:");
    let game: Option<Game> = db.select(("game", game_id_clean)).await?;
    let mut game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.dockerfile_path = Some(file_path.to_string_lossy().to_string());
    game.updated_at = Datetime::default();

    let _: Option<Game> = db.update(("game", game_id_clean)).content(game).await?;

    Ok(file_path.to_string_lossy().to_string())
}

pub async fn get_dockerfile_path(db: &Database, game_id: &str) -> ApiResult<String> {
    let game_id_clean = game_id.trim_start_matches("game:");
    let game: Option<Game> = db.select(("game", game_id_clean)).await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.dockerfile_path
        .ok_or_else(|| ApiError::NotFound("Dockerfile not found for this game".to_string()))
}
