use crate::app_state::AppState;
use crate::middleware::auth::Claims;
use crate::services::capacity::CapacityStats;
use crate::services::playground::PlaygroundRegistries;
use crate::services::storage::RoomMeta;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn health() -> &'static str {
    "OK"
}

pub async fn get_capacity(State(state): State<Arc<AppState>>) -> Json<CapacityStats> {
    Json(state.capacity.snapshot())
}

#[derive(Debug, Deserialize)]
pub struct ListRoomsQuery {
    pub game: Option<String>,
    pub phase: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PlaygroundStartRequest {
    pub game_id: String,
}

#[derive(Debug, Serialize)]
pub struct PlaygroundStartResponse {
    pub room_id: String,
    pub game_id: String,
    pub bot_player_id: String,
}

/// Create a fresh room for protocol-learning, attach a built-in sample
/// bot, and return the IDs the client uses to open the WebSocket.
pub async fn playground_start(
    Extension(_claims): Extension<Claims>,
    State(regs): State<PlaygroundRegistries>,
    Json(req): Json<PlaygroundStartRequest>,
) -> Result<Json<PlaygroundStartResponse>, StatusCode> {
    let host = regs
        .get(req.game_id.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let room_id = format!("playground-{}", uuid::Uuid::new_v4());
    let bot_pid = host.spawn(room_id.clone()).await.map_err(|e| {
        tracing::error!("playground spawn failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(PlaygroundStartResponse {
        room_id,
        game_id: req.game_id,
        bot_player_id: bot_pid,
    }))
}

/// Discovery endpoint. Reads from the `MetaIndex` maintained by the
/// lease holder of each loaded room. Open to anonymous callers.
pub async fn list_rooms(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListRoomsQuery>,
) -> Result<Json<Vec<RoomMeta>>, StatusCode> {
    let limit = q.limit.unwrap_or(100).min(500);
    state
        .meta
        .list(q.game.as_deref(), q.phase.as_deref(), limit)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("list_rooms failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
