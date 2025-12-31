use crate::{
    error::{ApiError, ApiResult},
    models::{GameResponse, GAMES, find_game_by_id},
};
use axum::{
    Json,
    extract::Path,
};

pub async fn get_game(Path(game_id): Path<String>) -> ApiResult<Json<GameResponse>> {
    find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))
        .map(|game| Json(game.clone()))
}

pub async fn list_games() -> Json<Vec<GameResponse>> {
    Json(GAMES.iter().cloned().collect())
}
