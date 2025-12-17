use crate::{
    AppState,
    error::ApiResult,
    models::{Claims, CreateGameRequest, GameResponse, UpdateGameRequest},
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use surrealdb::sql::Thing;
use validator::Validate;

pub async fn create_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateGameRequest>,
) -> ApiResult<(StatusCode, Json<GameResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let game = services::game::create_game(
        &state.db,
        payload.name,
        payload.description,
        payload.supported_languages,
        None, // Admin creates games without owner
    )
    .await?;
    Ok((StatusCode::CREATED, Json(game.into())))
}

pub async fn create_game_as_game_setter(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateGameRequest>,
) -> ApiResult<(StatusCode, Json<GameResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    // Get user from claims and set as owner
    let owner_id = Some(claims.sub.clone());

    let game = services::game::create_game(
        &state.db,
        payload.name,
        payload.description,
        payload.supported_languages,
        owner_id,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(game.into())))
}

pub async fn get_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> ApiResult<Json<GameResponse>> {
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let game = services::game::get_game(&state.db, game_id).await?;
    Ok(Json(game.into()))
}

pub async fn list_games(State(state): State<AppState>) -> ApiResult<Json<Vec<GameResponse>>> {
    let games = services::game::list_games(&state.db, true).await?;
    Ok(Json(games.into_iter().map(|g| g.into()).collect()))
}

pub async fn update_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(payload): Json<UpdateGameRequest>,
) -> ApiResult<Json<GameResponse>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let game = services::game::update_game(
        &state.db,
        game_id,
        payload.name,
        payload.description,
        payload.supported_languages,
        payload.is_active,
    )
    .await?;
    Ok(Json(game.into()))
}

pub async fn delete_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> ApiResult<StatusCode> {
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    services::game::delete_game(&state.db, game_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_my_games(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Vec<GameResponse>>> {
    let owner_id: Thing = claims
        .sub
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    let games = services::game::list_games_by_owner(&state.db, owner_id).await?;
    Ok(Json(games.into_iter().map(|g| g.into()).collect()))
}

pub async fn delete_game_as_game_setter(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<String>,
) -> ApiResult<StatusCode> {
    // Get the game to check ownership
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let game = services::game::get_game(&state.db, game_id.clone()).await?;

    // Verify ownership (owner or admin)
    let user_id = claims.sub.to_string();
    let is_owner = game
        .owner_id
        .as_ref()
        .map(|owner| owner.to_string() == user_id)
        .unwrap_or(false);

    if !is_owner && claims.role != crate::models::UserRole::Admin {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have permission to delete this game".to_string(),
        ));
    }

    services::game::delete_game(&state.db, game_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
