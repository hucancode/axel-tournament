use crate::middleware::auth::Claims;
use crate::models::game::Game;
use crate::models::room::{CreateRoomRequest, RoomResponse, ListRoomsQuery, RoomListItem};
use axum::http::StatusCode;
use axum::response::Json;
use axum::extract::{Path, Query, State};
use axum::Extension;
use std::sync::Arc;

/// Create a new room
pub async fn create_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    let user_id = claims.sub;
    
    tracing::info!(
        "HTTP_CREATE_ROOM_REQUEST name='{}' game_id={} host_id={}",
        request.name,
        request.game_id,
        user_id,
    );

    match state
        .room_manager
        .create_room(
            request.name,
            request.game_id,
            user_id, // Use authenticated user_id as host_id
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
    Extension(claims): Extension<Claims>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    let player_id = claims.sub;

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
