use crate::{
    error::ApiResult,
    models::{
        matches::{CreateMatchRequest, Match, UpdateMatchResultRequest},
    },
    services,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListMatchesQuery {
    pub tournament_id: Option<String>,
    pub game_id: Option<String>,
    pub user_id: Option<String>,
}

pub async fn create_match(
    State(state): State<AppState>,
    Json(payload): Json<CreateMatchRequest>,
) -> ApiResult<(StatusCode, Json<Match>)> {
    payload.validate().map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let match_data = services::matches::create_match(
        &state.db,
        payload.tournament_id,
        payload.game_id,
        payload.participant_submission_ids,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(match_data)))
}

pub async fn get_match(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> ApiResult<Json<Match>> {
    let match_data = services::matches::get_match(&state.db, &match_id).await?;
    Ok(Json(match_data))
}

pub async fn list_matches(
    State(state): State<AppState>,
    Query(query): Query<ListMatchesQuery>,
) -> ApiResult<Json<Vec<Match>>> {
    let matches = services::matches::list_matches(
        &state.db,
        query.tournament_id,
        query.game_id,
        query.user_id,
    )
    .await?;
    Ok(Json(matches))
}

pub async fn update_match_result(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
    Json(payload): Json<UpdateMatchResultRequest>,
) -> ApiResult<Json<Match>> {
    payload.validate().map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let match_data = services::matches::update_match_result(
        &state.db,
        &match_id,
        payload.status,
        payload.participants,
        payload.metadata,
        payload.started_at,
        payload.completed_at,
    )
    .await?;

    Ok(Json(match_data))
}
