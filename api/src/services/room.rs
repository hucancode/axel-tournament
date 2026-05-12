// Room CRUD + lifecycle for the human-vs-human flow.
//
// Two ways a room appears:
//   * Manual: a player calls `create_room` from the lobby UI. The
//     resulting room is unranked unless tied to an active tournament.
//   * Auto: the tournament matchmaker pairs registered players and
//     creates a ranked room they're allowed to join.
//
// Game state lives on the judge's WebSocket transport. This module only
// owns DB-side coordination (membership, ranked metadata, finish hooks
// for ELO + tournament finalization).

use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{
        game::find_game_by_id,
        room::{Room, RoomStatus},
        tournament::{TournamentKind, TournamentStatus},
    },
    services::elo,
};
use axel_core::repo::matches::MatchRepo;
use axel_core::repo::room::RoomRepo;
use axel_core::repo::tournament::TournamentRepo;
use surrealdb::types::{Datetime, RecordId};

#[allow(clippy::too_many_arguments)]
pub async fn create_room(
    db: &Database,
    host_id: RecordId,
    game_id: String,
    name: String,
    max_players: u32,
    tournament_id: Option<RecordId>,
    is_ranked: bool,
    allowed_user_ids: Vec<RecordId>,
    human_timeout_ms: Option<u32>,
    config: Option<serde_json::Value>,
) -> AppResult<Room> {
    let config = config.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    find_game_by_id(&game_id)
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))?;
    if !(2..=16).contains(&max_players) {
        return Err(AppError::Validation("Max players must be 2-16".to_string()));
    }

    let room = Room {
        id: None,
        game_id,
        host_id: host_id.clone(),
        name,
        max_players,
        status: RoomStatus::Lobby,
        players: vec![host_id],
        human_timeout_ms,
        tournament_id,
        allowed_user_ids,
        is_ranked,
        winner_id: None,
        event_history: Vec::new(),
        config,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    <Database as RoomRepo>::create(db, room).await
}

/// Create a ranked tournament room with a fixed participant list.
/// Players outside the allowed set will be rejected at `join_room`.
pub async fn create_ranked_room(
    db: &Database,
    tournament_id: RecordId,
    allowed_user_ids: Vec<RecordId>,
) -> AppResult<Room> {
    if allowed_user_ids.len() < 2 {
        return Err(AppError::Validation(
            "Ranked room needs at least 2 allowed players".to_string(),
        ));
    }
    let tournament = <Database as TournamentRepo>::get_by_id(db, &tournament_id).await?;
    if tournament.kind != TournamentKind::Human {
        return Err(AppError::BadRequest(
            "Only human tournaments produce ranked rooms".to_string(),
        ));
    }
    if tournament.status != TournamentStatus::Running {
        return Err(AppError::BadRequest(
            "Tournament must be running to spawn rooms".to_string(),
        ));
    }
    let host_id = allowed_user_ids[0].clone();
    let max = allowed_user_ids.len() as u32;
    create_room(
        db,
        host_id,
        tournament.game_id.clone(),
        format!("ranked-{}", uuid::Uuid::new_v4().simple()),
        max,
        Some(tournament_id),
        true,
        allowed_user_ids,
        None,
        Some(tournament.config.clone()),
    )
    .await
}

pub async fn get_room(db: &Database, room_id: RecordId) -> AppResult<Room> {
    <Database as RoomRepo>::get_by_id(db, &room_id).await
}

pub async fn list_open_rooms(db: &Database, game_id: Option<String>) -> AppResult<Vec<Room>> {
    <Database as RoomRepo>::list_open(db, game_id.as_deref()).await
}

pub async fn join_room(
    db: &Database,
    room_id: RecordId,
    user_id: RecordId,
) -> AppResult<Room> {
    let room = get_room(db, room_id.clone()).await?;
    if room.status != RoomStatus::Lobby {
        return Err(AppError::BadRequest(
            "Cannot join: room is not in lobby".to_string(),
        ));
    }
    if room.players.iter().any(|p| *p == user_id) {
        return Ok(room);
    }
    if room.players.len() as u32 >= room.max_players {
        return Err(AppError::BadRequest("Room is full".to_string()));
    }
    if !room.allowed_user_ids.is_empty()
        && !room.allowed_user_ids.iter().any(|p| *p == user_id)
    {
        return Err(AppError::Forbidden(
            "You are not invited to this ranked room".to_string(),
        ));
    }
    let mut resp = db
        .query(
            "UPDATE $rid SET players += $uid, updated_at = time::now()
             RETURN AFTER",
        )
        .bind(("rid", room_id))
        .bind(("uid", user_id))
        .await?;
    let rows: Vec<Room> = resp.take(0)?;
    rows.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Failed to join room".to_string()))
}

pub async fn leave_room(
    db: &Database,
    room_id: RecordId,
    user_id: RecordId,
) -> AppResult<()> {
    db.query("UPDATE $rid SET players -= $uid, updated_at = time::now()")
        .bind(("rid", room_id))
        .bind(("uid", user_id))
        .await?;
    Ok(())
}

/// Replace a room's per-game `config` blob. Only the host may edit, and
/// only while the room is still in lobby.
pub async fn update_room_config(
    db: &Database,
    room_id: RecordId,
    leader_id: RecordId,
    config: serde_json::Value,
) -> AppResult<Room> {
    let room = get_room(db, room_id.clone()).await?;
    if room.host_id != leader_id {
        return Err(AppError::Forbidden(
            "Only the room host can edit config".to_string(),
        ));
    }
    if room.status != RoomStatus::Lobby {
        return Err(AppError::BadRequest(
            "Cannot edit config after game has started".to_string(),
        ));
    }
    let mut resp = db
        .query(
            "UPDATE $rid SET config = $cfg, updated_at = time::now()
             RETURN AFTER",
        )
        .bind(("rid", room_id.clone()))
        .bind(("cfg", config))
        .await?;
    let rows: Vec<Room> = resp.take(0)?;
    rows.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Failed to update room config".to_string()))
}

pub async fn start_room(
    db: &Database,
    room_id: RecordId,
    leader_id: RecordId,
) -> AppResult<Room> {
    let room = get_room(db, room_id.clone()).await?;
    if room.host_id != leader_id {
        return Err(AppError::Forbidden(
            "Only the room host can start the game".to_string(),
        ));
    }
    if room.status != RoomStatus::Lobby {
        return Err(AppError::BadRequest("Room is not in lobby".to_string()));
    }
    if (room.players.len() as u32) < 2 {
        return Err(AppError::BadRequest(
            "Need at least 2 players to start".to_string(),
        ));
    }
    let mut resp = db
        .query(
            "UPDATE $rid SET status = 'playing', updated_at = time::now()
             RETURN AFTER",
        )
        .bind(("rid", room_id.clone()))
        .await?;
    let rows: Vec<Room> = resp.take(0)?;
    if let Some(room) = rows.into_iter().next() {
        return Ok(room);
    }
    get_room(db, room_id).await
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FinishReason {
    /// Played to a normal conclusion. winner_id may be None for draws.
    Played,
    /// One side disconnected and never returned. They lose by default;
    /// the winner is the surviving side.
    DisconnectTimeout,
}

/// Persist a finished room: write `match` row, set room state, and run
/// any tournament-side bookkeeping (ELO, finalization). Idempotent if
/// called twice with the same room_id (room status already 'finished').
///
/// `faulted_user_ids` lists players who lost by fault (e.g. timed out
/// missing their turn after disconnecting). A disconnect that the
/// player recovers from in time is NOT a fault — pass an empty vec.
///
/// `per_player_scores` (optional): for score-based games (poker chips,
/// rps round-wins, pd points), the judge can pass actual per-player
/// score deltas keyed by user_id. When `None`, defaults to 1.0/0.0/0.5
/// from `winner_id` (good for win/lose games).
pub async fn finish_room(
    db: &Database,
    room_id: RecordId,
    winner_id: Option<RecordId>,
    reason: FinishReason,
    faulted_user_ids: Vec<RecordId>,
    per_player_scores: Option<Vec<(RecordId, f64)>>,
) -> AppResult<Room> {
    use crate::models::matches::{Match, MatchParticipant, MatchStatus};

    let room = get_room(db, room_id.clone()).await?;
    if room.status == RoomStatus::Finished {
        return Ok(room);
    }

    let score_for = |uid: &RecordId| -> Option<f64> {
        if let Some(scores) = per_player_scores.as_ref() {
            return scores
                .iter()
                .find(|(u, _)| u == uid)
                .map(|(_, s)| *s)
                .or(Some(0.0));
        }
        Some(if Some(uid) == winner_id.as_ref() {
            1.0
        } else if winner_id.is_some() {
            0.0
        } else {
            0.5
        })
    };

    let participants = room
        .players
        .iter()
        .map(|uid| MatchParticipant {
            user_id: uid.clone(),
            submission_id: None,
            score: score_for(uid),
        })
        .collect::<Vec<_>>();

    let error_message = match reason {
        FinishReason::DisconnectTimeout => Some("disconnect_timeout".to_string()),
        FinishReason::Played => None,
    };

    let m = Match {
        tournament_id: room.tournament_id.clone(),
        game_id: room.game_id.clone(),
        status: MatchStatus::Completed,
        participants,
        room_id: Some(room_id.clone()),
        error_message,
        faulted_user_ids,
        started_at: room.created_at.clone().into(),
        completed_at: Some(Datetime::default()),
        ..Default::default()
    };
    <Database as MatchRepo>::create(db, m).await?;

    let mut resp = db
        .query(
            "UPDATE $rid SET status = 'finished', winner_id = $w,
                              updated_at = time::now()
             RETURN AFTER",
        )
        .bind(("rid", room_id))
        .bind(("w", winner_id.clone()))
        .await?;
    let rows: Vec<Room> = resp.take(0)?;
    let updated = rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Failed to finish room".to_string()))?;

    let Some(tid) = updated.tournament_id.clone().filter(|_| updated.is_ranked) else {
        return Ok(updated);
    };
    // Score-aligned-by-player-index for elo's Score path.
    let aligned: Option<Vec<f64>> = per_player_scores.as_ref().map(|ps| {
        updated
            .players
            .iter()
            .map(|uid| {
                ps.iter()
                    .find(|(u, _)| u == uid)
                    .map(|(_, s)| *s)
                    .unwrap_or(0.0)
            })
            .collect()
    });
    elo::apply_ranked_result(
        db,
        &tid,
        &updated.game_id,
        &updated.players,
        &winner_id,
        aligned.as_deref(),
    )
    .await?;
    if let Err(e) = crate::services::finalization::finalize_if_done(db, tid).await {
        tracing::warn!("finalize_if_done after ranked room: {e}");
    }
    Ok(updated)
}
