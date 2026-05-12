use crate::{
    AppState,
    error::AppResult,
    models::{
        matches::{CreateMatchRequest, MatchResponse},
        rid,
    },
    services,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListMatchesQuery {
    pub tournament_id: Option<String>,
    pub game_id: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn create_match(
    State(state): State<AppState>,
    Json(payload): Json<CreateMatchRequest>,
) -> AppResult<(StatusCode, Json<MatchResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
    let tournament_id = rid("tournament", payload.tournament_id);
    let game_id = payload.game_id;
    let submission_ids = payload
        .participant_submission_ids
        .into_iter()
        .map(|id| rid("submission", id))
        .collect::<Vec<_>>();
    let match_data =
        services::matches::create_match(&state.db, tournament_id, game_id, submission_ids).await?;
    Ok((StatusCode::CREATED, Json(match_data.into())))
}

pub async fn get_match(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> AppResult<Json<MatchResponse>> {
    let match_data = services::matches::get_match(&state.db, rid("match", match_id)).await?;
    Ok(Json(match_data.into()))
}

pub async fn list_matches(
    State(state): State<AppState>,
    Query(query): Query<ListMatchesQuery>,
) -> AppResult<Json<Vec<MatchResponse>>> {
    let tournament_id = query.tournament_id.map(|id| rid("tournament", id));
    let game_id = query.game_id.map(|id| rid("game", id));
    let user_id = query.user_id.map(|id| rid("user", id));
    let matches = services::matches::list_matches(
        &state.db,
        tournament_id,
        game_id,
        user_id,
        query.limit,
        query.offset,
    )
    .await?;
    Ok(Json(matches.into_iter().map(Into::into).collect()))
}
