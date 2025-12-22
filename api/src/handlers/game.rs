use crate::{
    AppState,
    error::{ApiError, ApiResult},
    models::{Claims, CreateGameRequest, GameResponse, ProgrammingLanguage, UpdateGameRequest, UserRole},
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use surrealdb::sql::Thing;
use validator::Validate;

fn validate_game_code_fields(
    game_code: &Option<String>,
    game_language: &Option<ProgrammingLanguage>,
) -> ApiResult<()> {
    if game_code.is_some() != game_language.is_some() {
        return Err(ApiError::Validation(
            "game_code and game_language must be provided together".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_game_owner(state: &AppState, game_id: Thing, claims: &Claims) -> ApiResult<()> {
    if claims.role == UserRole::Admin {
        return Ok(());
    }
    let game = services::game::get_game(&state.db, game_id).await?;
    if game.owner_id.to_string() != claims.sub {
        return Err(ApiError::Forbidden(
            "You don't have permission to update this game".to_string(),
        ));
    }
    Ok(())
}

pub async fn create_game(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
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
        claims.sub,
        payload.game_code,
        payload.game_language,
        payload.rounds_per_match,
        payload.repetitions,
        payload.timeout_ms,
        payload.cpu_limit,
        payload.turn_timeout_ms,
        payload.memory_limit_mb,
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
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<String>,
    Json(payload): Json<UpdateGameRequest>,
) -> ApiResult<Json<GameResponse>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    validate_game_code_fields(&payload.game_code, &payload.game_language)?;
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    ensure_game_owner(&state, game_id.clone(), &claims).await?;
    let game = services::game::update_game(
        &state.db,
        game_id,
        payload.name,
        payload.description,
        payload.supported_languages,
        payload.is_active,
        payload.game_code,
        payload.game_language,
        payload.rounds_per_match,
        payload.repetitions,
        payload.timeout_ms,
        payload.cpu_limit,
        payload.turn_timeout_ms,
        payload.memory_limit_mb,
    )
    .await?;
    Ok(Json(game.into()))
}

pub async fn delete_game(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<String>,
) -> ApiResult<StatusCode> {
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    ensure_game_owner(&state, game_id.clone(), &claims).await?;
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
