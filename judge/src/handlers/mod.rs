use crate::app_state::AppState;
use crate::db::Database;
use crate::middleware::auth::Claims;
use crate::models::game_metadata::find_game_by_id;
use crate::services::capacity::CapacityStats;
use crate::services::playground::PlaygroundRegistries;
use crate::services::playground_submission::SubmissionPlaygroundRegistries;
use crate::services::storage::RoomMeta;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

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

/// Composite state for the "play against your own submission"
/// playground endpoint: registries dispatch per game; the DB
/// authorizes ownership + binary lookup.
#[derive(Clone)]
pub struct SubmissionPlaygroundState {
    pub registries: SubmissionPlaygroundRegistries,
    pub db: Database,
}

#[derive(Debug, Deserialize)]
pub struct PlaygroundStartWithSubmissionRequest {
    pub game_id: String,
    pub submission_id: String,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct OwnedSubmissionRow {
    #[serde(default)]
    pub user_id: Option<RecordId>,
    #[serde(default)]
    pub game_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub compiled_binary_path: Option<String>,
}

/// Spawn the caller's compiled bot into a fresh playground room and
/// return the IDs the client uses to open the WebSocket. The bot is
/// the opponent — the human caller is the other player.
pub async fn playground_start_with_submission(
    Extension(claims): Extension<Claims>,
    State(state): State<SubmissionPlaygroundState>,
    Json(req): Json<PlaygroundStartWithSubmissionRequest>,
) -> Result<Json<PlaygroundStartResponse>, (StatusCode, String)> {
    let game = find_game_by_id(&req.game_id)
        .ok_or((StatusCode::BAD_REQUEST, "unknown game".into()))?;
    let host = state.registries.get(game.id).ok_or((
        StatusCode::BAD_REQUEST,
        "game does not support submission playground".into(),
    ))?;

    let sid = RecordId::new("submission", req.submission_id.as_str());
    let mut resp = state
        .db
        .query(
            "SELECT user_id, game_id, status, compiled_binary_path
             FROM $sid;",
        )
        .bind(("sid", sid.clone()))
        .await
        .map_err(|e| {
            tracing::error!("submission lookup failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "db error".into())
        })?;
    let rows: Vec<OwnedSubmissionRow> = resp.take(0).map_err(|e| {
        tracing::error!("submission decode failed: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "db decode".into())
    })?;
    let row = rows.into_iter().next().ok_or((
        StatusCode::NOT_FOUND,
        "submission not found".into(),
    ))?;

    let owner = row
        .user_id
        .as_ref()
        .ok_or((StatusCode::FORBIDDEN, "submission has no owner".into()))?;
    if owner.to_sql() != claims.sub {
        return Err((StatusCode::FORBIDDEN, "not your submission".into()));
    }
    if row.game_id.as_deref() != Some(game.id) {
        return Err((
            StatusCode::BAD_REQUEST,
            "submission is for a different game".into(),
        ));
    }
    if row.status.as_deref() != Some("accepted") {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "submission not accepted (status={})",
                row.status.as_deref().unwrap_or("unknown")
            ),
        ));
    }
    let binary_path = row
        .compiled_binary_path
        .filter(|p| !p.is_empty())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "submission has no compiled binary".into(),
        ))?;

    let turn_timeout = Duration::from_millis(game.bot_turn_timeout_ms);
    let (room_id, bot_pid) = host
        .spawn(
            req.submission_id.clone(),
            PathBuf::from(binary_path),
            turn_timeout,
        )
        .await
        .map_err(|e| {
            tracing::error!("submission playground spawn failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "spawn failed".into())
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
