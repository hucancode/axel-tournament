use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use surrealdb::sql::Thing;
use validator::Validate;

use crate::{
    error::ApiResult,
    models::{
        Claims, CreateRoomRequest, CreateRoomMessageRequest, RoomResponse, 
        RoomMessageResponse,
    },
    services, AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListRoomsQuery {
    pub game_id: Option<String>,
}

pub async fn create_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateRoomRequest>,
) -> ApiResult<(StatusCode, Json<RoomResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let room = services::room::create_room(
        &state.db,
        payload.game_id,
        claims.sub,
        payload.name,
        payload.max_players,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(room.into())))
}

pub async fn get_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid room id".to_string()))?;

    let room = services::room::get_room(&state.db, room_thing).await?;
    Ok(Json(room.into()))
}

pub async fn list_rooms(
    State(state): State<AppState>,
    Query(query): Query<ListRoomsQuery>,
) -> ApiResult<Json<Vec<RoomResponse>>> {
    let rooms = services::room::list_rooms(&state.db, query.game_id).await?;
    let responses: Vec<RoomResponse> = rooms.into_iter().map(|r| r.into()).collect();
    Ok(Json(responses))
}

pub async fn join_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid room id".to_string()))?;

    let room = services::room::join_room(&state.db, room_thing, claims.sub).await?;
    Ok(Json(room.into()))
}

pub async fn leave_room(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid room id".to_string()))?;

    let room = services::room::leave_room(&state.db, room_thing, claims.sub).await?;
    Ok(Json(room.into()))
}

pub async fn start_game(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid room id".to_string()))?;

    let room = services::room::start_game(&state.db, room_thing, claims.sub).await?;
    Ok(Json(room.into()))
}

pub async fn create_room_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<String>,
    Json(payload): Json<CreateRoomMessageRequest>,
) -> ApiResult<(StatusCode, Json<RoomMessageResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let message = services::room::create_room_message(
        &state.db,
        room_id,
        claims.sub,
        payload.message,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(message.into())))
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<u32>,
}

pub async fn get_room_messages(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Query(query): Query<GetMessagesQuery>,
) -> ApiResult<Json<Vec<RoomMessageResponse>>> {
    let messages = services::room::get_room_messages(&state.db, room_id, query.limit).await?;
    let responses: Vec<RoomMessageResponse> = messages.into_iter().map(|m| m.into()).collect();
    Ok(Json(responses))
}
