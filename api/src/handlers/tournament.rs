use crate::{
    AppState,
    error::ApiResult,
    models::{
        Claims, CreateTournamentRequest, TournamentParticipant, TournamentResponse, TournamentStatus,
        UpdateTournamentRequest, UserRole,
    },
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use surrealdb::sql::Thing;
use validator::Validate;

async fn ensure_game_owner(
    state: &AppState,
    game_id: Thing,
    claims: &Claims,
) -> ApiResult<()> {
    if claims.role == UserRole::Admin {
        return Ok(());
    }
    let game = services::game::get_game(&state.db, game_id).await?;
    let is_owner = game.owner_id.to_string() == claims.sub;
    if !is_owner {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have permission to manage tournaments for this game".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_tournament_owner(
    state: &AppState,
    tournament_id: Thing,
    claims: &Claims,
) -> ApiResult<()> {
    if claims.role == UserRole::Admin {
        return Ok(());
    }
    let tournament = services::tournament::get_tournament(&state.db, tournament_id).await?;
    ensure_game_owner(state, tournament.game_id, claims).await
}

pub async fn create_tournament(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateTournamentRequest>,
) -> ApiResult<(StatusCode, Json<TournamentResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    if payload.min_players > payload.max_players {
        return Err(crate::error::ApiError::Validation(
            "min_players cannot be greater than max_players".to_string(),
        ));
    }
    let game_id: Thing = payload
        .game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    if claims.role == UserRole::GameSetter {
        ensure_game_owner(&state, game_id.clone(), &claims).await?;
    }
    let tournament = services::tournament::create_tournament(
        &state.db,
        game_id,
        payload.name,
        payload.description,
        payload.min_players,
        payload.max_players,
        payload.start_time,
        payload.end_time,
        payload.match_generation_type,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(tournament.into())))
}

pub async fn get_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<TournamentResponse>> {
    let tournament = services::tournament::get_tournament(
        &state.db,
        tournament_id
            .parse()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
    )
    .await?;
    Ok(Json(tournament.into()))
}

#[derive(Deserialize)]
pub struct ListTournamentsQuery {
    status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn list_tournaments(
    State(state): State<AppState>,
    Query(query): Query<ListTournamentsQuery>,
) -> ApiResult<Json<Vec<TournamentResponse>>> {
    let status = if let Some(status_str) = query.status {
        Some(
            serde_json::from_str::<TournamentStatus>(&format!("\"{}\"", status_str))
                .map_err(|_| crate::error::ApiError::Validation("Invalid status".to_string()))?,
        )
    } else {
        None
    };
    let tournaments =
        services::tournament::list_tournaments(&state.db, status, query.limit, query.offset)
            .await?;
    let response: Vec<TournamentResponse> = tournaments.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn update_tournament(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tournament_id): Path<String>,
    Json(payload): Json<UpdateTournamentRequest>,
) -> ApiResult<Json<TournamentResponse>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let tournament_id: Thing = tournament_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?;
    if claims.role == UserRole::GameSetter {
        ensure_tournament_owner(&state, tournament_id.clone(), &claims).await?;
    }
    let tournament = services::tournament::update_tournament(
        &state.db,
        tournament_id,
        payload.name,
        payload.description,
        payload.status,
        payload.start_time,
        payload.end_time,
    )
    .await?;
    Ok(Json(tournament.into()))
}

pub async fn join_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<(StatusCode, Json<TournamentParticipant>)> {
    let participant = services::tournament::join_tournament(
        &state.db,
        tournament_id
            .parse()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
        claims
            .sub
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(participant)))
}

pub async fn leave_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<StatusCode> {
    services::tournament::leave_tournament(
        &state.db,
        tournament_id
            .parse()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
        claims
            .sub
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_tournament_participants(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<Vec<TournamentParticipant>>> {
    let participants = services::tournament::get_tournament_participants(
        &state.db,
        tournament_id
            .parse()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
    )
    .await?;
    Ok(Json(participants))
}

/// Start a tournament and generate matches (admin only)
pub async fn start_tournament(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tournament_id): Path<String>,
) -> ApiResult<Json<TournamentResponse>> {
    let tournament_id: Thing = tournament_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?;
    if claims.role == UserRole::GameSetter {
        ensure_tournament_owner(&state, tournament_id.clone(), &claims).await?;
    }
    let tournament = services::tournament::start_tournament(
        &state.db,
        tournament_id,
    )
    .await?;
    Ok(Json(tournament.into()))
}
