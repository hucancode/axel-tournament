use crate::auth::validate_jwt;
use crate::games::Game;
use crate::room::models::{CreateRoomRequest, RoomResponse, ListRoomsQuery, RoomListItem};
use axum::http::StatusCode;
use axum::response::Json;
use axum::extract::{Path, Query, State};
use std::sync::Arc;

/// Create a new room
pub async fn create_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    tracing::info!(
        "HTTP_CREATE_ROOM_REQUEST name='{}' game_id={} host_id={}",
        request.name,
        request.game_id,
        request.host_id,
    );

    match state
        .room_manager
        .create_room(
            request.name,
            request.game_id,
            request.host_id,
            state.game.max_players(),
            request.human_timeout_ms,
        )
        .await
    {
        Ok(room) => {
            tracing::info!("HTTP_CREATE_ROOM_SUCCESS room_id={}", room.id);
            Ok(Json(room.to_response()))
        }
        Err(e) => {
            tracing::error!("HTTP_CREATE_ROOM_FAILED error={}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get room details
pub async fn get_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    if let Some(room) = state.room_manager.get_room(&room_id).await {
        Ok(Json(room))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Join a room
pub async fn join_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    // Extract JWT from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let player_id =
        validate_jwt(auth_header, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    match state.room_manager.join_room(&room_id, player_id).await {
        Ok((response, _is_reconnecting)) => Ok(Json(response)),
        Err(e) => {
            tracing::warn!("Failed to join room: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// List all waiting rooms
pub async fn list_rooms<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Query(query): Query<ListRoomsQuery>,
) -> Json<Vec<RoomListItem>>
where
    G: Game + Clone + Send + Sync + 'static,
{
    let rooms = state
        .room_manager
        .list_rooms(query.game_id.as_deref())
        .await;
    Json(rooms)
}
