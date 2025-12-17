use crate::{
    AppState,
    error::ApiResult,
    models::{CreateMatchPolicyRequest, MatchPolicy, UpdateMatchPolicyRequest},
    services,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use surrealdb::sql::Thing;
use validator::Validate;

pub async fn create_policy(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Json(payload): Json<CreateMatchPolicyRequest>,
) -> ApiResult<(StatusCode, Json<MatchPolicy>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let policy = services::match_policy::create_policy(
        &state.db,
        tournament_id
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
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
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<MatchPolicy>> {
    let policy = services::match_policy::get_policy(
        &state.db,
        tournament_id
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
    )
    .await?;
    Ok(Json(policy))
}

pub async fn update_policy(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Json(payload): Json<UpdateMatchPolicyRequest>,
) -> ApiResult<Json<MatchPolicy>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let policy = services::match_policy::update_policy(
        &state.db,
        tournament_id
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
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
