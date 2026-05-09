use crate::app_state::AppState;
use crate::services::capacity::CapacityStats;
use crate::services::storage::RoomMeta;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

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
