use crate::{
    AppState,
    error::ApiResult,
    models::{
        Claims, CreateTournamentRequest, Tournament, TournamentParticipant, TournamentStatus,
        UpdateTournamentRequest,
    },
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use validator::Validate;

pub async fn create_tournament(
    State(state): State<AppState>,
    Json(payload): Json<CreateTournamentRequest>,
) -> ApiResult<(StatusCode, Json<Tournament>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    if payload.min_players > payload.max_players {
        return Err(crate::error::ApiError::Validation(
            "min_players cannot be greater than max_players".to_string(),
        ));
    }
    let tournament = services::tournament::create_tournament(
        &state.db,
        payload.game_id,
        payload.name,
        payload.description,
        payload.min_players,
        payload.max_players,
        payload.start_time,
        payload.end_time,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(tournament)))
}

pub async fn get_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<Tournament>> {
    let tournament = services::tournament::get_tournament(&state.db, &tournament_id).await?;
    Ok(Json(tournament))
}

#[derive(Deserialize)]
pub struct ListTournamentsQuery {
    status: Option<String>,
}

pub async fn list_tournaments(
    State(state): State<AppState>,
    Query(query): Query<ListTournamentsQuery>,
) -> ApiResult<Json<Vec<Tournament>>> {
    let status = if let Some(status_str) = query.status {
        Some(
            serde_json::from_str::<TournamentStatus>(&format!("\"{}\"", status_str))
                .map_err(|_| crate::error::ApiError::Validation("Invalid status".to_string()))?,
        )
    } else {
        None
    };
    let tournaments = services::tournament::list_tournaments(&state.db, status).await?;
    Ok(Json(tournaments))
}

pub async fn update_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Json(payload): Json<UpdateTournamentRequest>,
) -> ApiResult<Json<Tournament>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let tournament = services::tournament::update_tournament(
        &state.db,
        &tournament_id,
        payload.name,
        payload.description,
        payload.status,
        payload.start_time,
        payload.end_time,
    )
    .await?;
    Ok(Json(tournament))
}

pub async fn join_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<(StatusCode, Json<TournamentParticipant>)> {
    let participant =
        services::tournament::join_tournament(&state.db, &tournament_id, &claims.sub).await?;
    Ok((StatusCode::CREATED, Json(participant)))
}

pub async fn leave_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<StatusCode> {
    services::tournament::leave_tournament(&state.db, &tournament_id, &claims.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_tournament_participants(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<Vec<TournamentParticipant>>> {
    let participants =
        services::tournament::get_tournament_participants(&state.db, &tournament_id).await?;
    Ok(Json(participants))
}
