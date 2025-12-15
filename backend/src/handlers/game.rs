use crate::{
    AppState,
    error::ApiResult,
    models::{CreateGameRequest, Game, UpdateGameRequest},
    services,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use validator::Validate;

pub async fn create_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateGameRequest>,
) -> ApiResult<(StatusCode, Json<Game>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let game = services::game::create_game(
        &state.db,
        payload.name,
        payload.description,
        payload.rules,
        payload.supported_languages,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(game)))
}

pub async fn get_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> ApiResult<Json<Game>> {
    let game = services::game::get_game(&state.db, &game_id).await?;
    Ok(Json(game))
}

pub async fn list_games(State(state): State<AppState>) -> ApiResult<Json<Vec<Game>>> {
    let games = services::game::list_games(&state.db, true).await?;
    Ok(Json(games))
}

pub async fn update_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(payload): Json<UpdateGameRequest>,
) -> ApiResult<Json<Game>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let game = services::game::update_game(
        &state.db,
        &game_id,
        payload.name,
        payload.description,
        payload.rules,
        payload.supported_languages,
        payload.is_active,
    )
    .await?;
    Ok(Json(game))
}

pub async fn delete_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> ApiResult<StatusCode> {
    services::game::delete_game(&state.db, &game_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
