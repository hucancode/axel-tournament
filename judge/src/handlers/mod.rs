use crate::app_state::AppState;
use crate::games::{Pd, Rps, Ttt};
use crate::middleware::auth::Claims;
use crate::services::capacity::CapacityStats;
use crate::services::room::logic::RoomRegistry;
use crate::services::room::playground;
use crate::services::storage::RoomMeta;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Registries shared with the playground handler. Same instances the
/// WebSocket handler uses, so playground rooms ride the normal
/// JOIN/START/MOVE plumbing.
#[derive(Clone)]
pub struct PlaygroundRegistries {
    pub rps: Arc<RoomRegistry<Rps>>,
    pub ttt: Arc<RoomRegistry<Ttt>>,
    pub pd: Arc<RoomRegistry<Pd>>,
}

pub async fn health() -> &'static str {
    "OK"
}

pub async fn get_capacity(State(state): State<Arc<AppState>>) -> Json<CapacityStats> {
    let stats = state.capacity.get_stats().await;
    Json(stats)
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
/// The bot waits for the human to JOIN before joining itself, so the
/// human becomes host and triggers `START`.
pub async fn playground_start(
    Extension(_claims): Extension<Claims>,
    State(regs): State<PlaygroundRegistries>,
    Json(req): Json<PlaygroundStartRequest>,
) -> Result<Json<PlaygroundStartResponse>, StatusCode> {
    let room_id = format!("playground-{}", uuid::Uuid::new_v4());
    let bot_pid = match req.game_id.as_str() {
        "rock-paper-scissors" => playground::spawn_rps(regs.rps.clone(), room_id.clone())
            .await
            .map_err(|e| {
                tracing::error!("playground rps spawn failed: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
        "tic-tac-toe" => playground::spawn_ttt(regs.ttt.clone(), room_id.clone())
            .await
            .map_err(|e| {
                tracing::error!("playground ttt spawn failed: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
        "prisoners-dilemma" => playground::spawn_pd(regs.pd.clone(), room_id.clone())
            .await
            .map_err(|e| {
                tracing::error!("playground pd spawn failed: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    Ok(Json(PlaygroundStartResponse {
        room_id,
        game_id: req.game_id,
        bot_player_id: bot_pid,
    }))
}

/// Discovery endpoint. Reads from the `MetaIndex` maintained by the
/// lease holder of each loaded room. Open to anonymous callers — only
/// metadata, no auth required.
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
