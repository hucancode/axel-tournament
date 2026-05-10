use crate::{
    AppState,
    error::ApiResult,
    models::{Claims, CreateRoomRequest, RoomResponse, rid},
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use surrealdb::types::RecordId;
use validator::Validate;

#[derive(Deserialize)]
pub struct ListRoomsQuery {
    pub game_id: Option<String>,
}

pub async fn list_rooms(
    State(state): State<AppState>,
    Query(q): Query<ListRoomsQuery>,
) -> ApiResult<Json<Vec<RoomResponse>>> {
    let rooms = services::room::list_open_rooms(&state.db, q.game_id).await?;
    Ok(Json(rooms.into_iter().map(Into::into).collect()))
}

pub async fn create_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateRoomRequest>,
) -> ApiResult<(StatusCode, Json<RoomResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let host_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    let tournament_record = payload.tournament_id.map(|tid| rid("tournament", tid));
    let room = services::room::create_room(
        &state.db,
        host_id,
        payload.game_id,
        payload.name,
        payload.max_players,
        tournament_record,
        false,
        Vec::new(),
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(room.into())))
}

pub async fn join_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    let room = services::room::join_room(&state.db, rid("room", room_id), user_id).await?;
    Ok(Json(room.into()))
}

pub async fn leave_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
) -> ApiResult<StatusCode> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    services::room::leave_room(&state.db, rid("room", room_id), user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    let room = services::room::start_room(&state.db, rid("room", room_id), user_id).await?;
    Ok(Json(room.into()))
}

#[derive(Deserialize)]
pub struct EnqueueRequest {
    pub tournament_id: String,
}

pub async fn enqueue_match(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<EnqueueRequest>,
) -> ApiResult<StatusCode> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    services::matchmaking::enqueue(&state.db, rid("tournament", payload.tournament_id), user_id)
        .await?;
    Ok(StatusCode::ACCEPTED)
}

pub async fn dequeue_match(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<EnqueueRequest>,
) -> ApiResult<StatusCode> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?;
    services::matchmaking::dequeue(&state.db, rid("tournament", payload.tournament_id), user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
