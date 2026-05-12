use crate::{
    error::{AppError, AppResult},
    models::{GAMES, GameMetadata, find_game_by_id},
};
use axum::{
    Json,
    extract::Path,
};

pub async fn get_game(Path(game_id): Path<String>) -> AppResult<Json<GameMetadata>> {
    find_game_by_id(&game_id)
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))
        .map(|game| Json(game.clone()))
}

pub async fn list_games() -> Json<Vec<GameMetadata>> {
    Json(GAMES.iter().cloned().collect())
}
