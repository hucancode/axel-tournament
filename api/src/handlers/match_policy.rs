use crate::{
    AppState,
    error::ApiResult,
    models::{Claims, CreateMatchPolicyRequest, MatchPolicy, UpdateMatchPolicyRequest, UserRole},
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use surrealdb::sql::Thing;
use validator::Validate;

async fn ensure_tournament_owner(
    state: &AppState,
    tournament_id: Thing,
    claims: &Claims,
) -> ApiResult<()> {
    if claims.role == UserRole::Admin {
        return Ok(());
    }
    let tournament = services::tournament::get_tournament(&state.db, tournament_id).await?;
    let game = services::game::get_game(&state.db, tournament.game_id).await?;
    let is_owner = game
        .owner_id
        .as_ref()
        .map(|owner| owner.to_string() == claims.sub)
        .unwrap_or(false);
    if !is_owner {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have permission to manage match policies for this tournament".to_string(),
        ));
    }
    Ok(())
}

pub async fn create_policy(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tournament_id): Path<String>,
    Json(payload): Json<CreateMatchPolicyRequest>,
) -> ApiResult<(StatusCode, Json<MatchPolicy>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let tournament_id: Thing = tournament_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?;
    ensure_tournament_owner(&state, tournament_id.clone(), &claims).await?;
    let policy = services::match_policy::create_policy(
        &state.db,
        tournament_id,
        payload.rounds_per_match.unwrap_or(1),
        payload.repetitions.unwrap_or(1),
        payload.timeout_seconds.unwrap_or(300),
        payload.cpu_limit,
        payload.memory_limit,
        payload.scoring_weights,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(policy)))
}

pub async fn get_policy(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<MatchPolicy>> {
    let tournament_id: Thing = tournament_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?;
    ensure_tournament_owner(&state, tournament_id.clone(), &claims).await?;
    let policy = services::match_policy::get_policy(&state.db, tournament_id).await?;
    Ok(Json(policy))
}

pub async fn update_policy(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tournament_id): Path<String>,
    Json(payload): Json<UpdateMatchPolicyRequest>,
) -> ApiResult<Json<MatchPolicy>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let tournament_id: Thing = tournament_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?;
    ensure_tournament_owner(&state, tournament_id.clone(), &claims).await?;
    let policy = services::match_policy::update_policy(
        &state.db,
        tournament_id,
        payload.rounds_per_match,
        payload.repetitions,
        payload.timeout_seconds,
        payload.cpu_limit,
        payload.memory_limit,
        payload.scoring_weights,
    )
    .await?;

    Ok(Json(policy))
}
