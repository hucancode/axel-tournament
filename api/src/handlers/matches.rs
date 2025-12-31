use crate::{
    AppState,
    error::ApiResult,
    models::matches::{CreateMatchRequest, MatchResponse},
    services,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use surrealdb::sql::Thing;
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
) -> ApiResult<(StatusCode, Json<MatchResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let tournament_id: Thing = payload
        .tournament_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?;
    let game_id = payload.game_id; // game_id is now a String
    let submission_ids = payload
        .participant_submission_ids
        .iter()
        .map(|id| {
            id.parse::<Thing>().map_err(|_| {
                crate::error::ApiError::BadRequest("Invalid submission id".to_string())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let match_data =
        services::matches::create_match(&state.db, tournament_id, game_id, submission_ids).await?;
    Ok((StatusCode::CREATED, Json(match_data.into())))
}

pub async fn get_match(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> ApiResult<Json<MatchResponse>> {
    let match_data = services::matches::get_match(
        &state.db,
        match_id
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid match id".to_string()))?,
    )
    .await?;
    Ok(Json(match_data.into()))
}

pub async fn list_matches(
    State(state): State<AppState>,
    Query(query): Query<ListMatchesQuery>,
) -> ApiResult<Json<Vec<MatchResponse>>> {
    let tournament_id = query
        .tournament_id
        .as_deref()
        .map(|id| {
            id.parse::<Thing>().map_err(|_| {
                crate::error::ApiError::BadRequest("Invalid tournament id".to_string())
            })
        })
        .transpose()?;
    let game_id = query
        .game_id
        .as_deref()
        .map(|id| {
            id.parse::<Thing>()
                .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))
        })
        .transpose()?;
    let user_id = query
        .user_id
        .as_deref()
        .map(|id| {
            id.parse::<Thing>()
                .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))
        })
        .transpose()?;
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
