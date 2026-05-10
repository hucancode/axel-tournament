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
    error::{ApiError, ApiResult},
    models::{
        game::find_game_by_id,
        room::{Room, RoomStatus},
        tournament::{Tournament, TournamentKind, TournamentStatus},
    },
    services::elo,
};
use surrealdb::types::{Datetime, RecordId};

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
) -> ApiResult<Room> {
    find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;
    if !(2..=16).contains(&max_players) {
        return Err(ApiError::Validation("Max players must be 2-16".to_string()));
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
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Room> = db.create("room").content(room).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create room".to_string()))
}

/// Create a ranked tournament room with a fixed participant list.
/// Players outside the allowed set will be rejected at `join_room`.
pub async fn create_ranked_room(
    db: &Database,
    tournament_id: RecordId,
    allowed_user_ids: Vec<RecordId>,
) -> ApiResult<Room> {
    if allowed_user_ids.len() < 2 {
        return Err(ApiError::Validation(
            "Ranked room needs at least 2 allowed players".to_string(),
        ));
    }
    let tournament: Tournament = {
        let opt: Option<Tournament> = db.select(&tournament_id).await?;
        opt.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))?
    };
    if tournament.kind != TournamentKind::Human {
        return Err(ApiError::BadRequest(
            "Only human tournaments produce ranked rooms".to_string(),
        ));
    }
    if tournament.status != TournamentStatus::Running {
        return Err(ApiError::BadRequest(
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
    )
    .await
}

pub async fn get_room(db: &Database, room_id: RecordId) -> ApiResult<Room> {
    let r: Option<Room> = db.select(&room_id).await?;
    r.ok_or_else(|| ApiError::NotFound("Room not found".to_string()))
}

pub async fn list_open_rooms(db: &Database, game_id: Option<String>) -> ApiResult<Vec<Room>> {
    let q = if let Some(gid) = game_id {
        db.query(
            "SELECT * FROM room WHERE status = 'lobby' AND game_id = $game_id
             ORDER BY created_at DESC LIMIT 200",
        )
        .bind(("game_id", gid))
    } else {
        db.query(
            "SELECT * FROM room WHERE status = 'lobby'
             ORDER BY created_at DESC LIMIT 200",
        )
    };
    let mut resp = q.await?;
    let rooms: Vec<Room> = resp.take(0)?;
    Ok(rooms)
}

pub async fn join_room(
    db: &Database,
    room_id: RecordId,
    user_id: RecordId,
) -> ApiResult<Room> {
    let room = get_room(db, room_id.clone()).await?;
    if room.status != RoomStatus::Lobby {
        return Err(ApiError::BadRequest(
            "Cannot join: room is not in lobby".to_string(),
        ));
    }
    if room.players.iter().any(|p| *p == user_id) {
        return Ok(room);
    }
    if room.players.len() as u32 >= room.max_players {
        return Err(ApiError::BadRequest("Room is full".to_string()));
    }
    if !room.allowed_user_ids.is_empty()
        && !room.allowed_user_ids.iter().any(|p| *p == user_id)
    {
        return Err(ApiError::Forbidden(
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
        .ok_or_else(|| ApiError::Internal("Failed to join room".to_string()))
}

pub async fn leave_room(
    db: &Database,
    room_id: RecordId,
    user_id: RecordId,
) -> ApiResult<()> {
    db.query("UPDATE $rid SET players -= $uid, updated_at = time::now()")
        .bind(("rid", room_id))
        .bind(("uid", user_id))
        .await?;
    Ok(())
}

pub async fn start_room(
    db: &Database,
    room_id: RecordId,
    leader_id: RecordId,
) -> ApiResult<Room> {
    let room = get_room(db, room_id.clone()).await?;
    if room.host_id != leader_id {
        return Err(ApiError::Forbidden(
            "Only the room host can start the game".to_string(),
        ));
    }
    if room.status != RoomStatus::Lobby {
        return Err(ApiError::BadRequest("Room is not in lobby".to_string()));
    }
    if (room.players.len() as u32) < 2 {
        return Err(ApiError::BadRequest(
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
) -> ApiResult<Room> {
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
    let _: Option<Match> = db.create("match").content(m).await?;

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
        .ok_or_else(|| ApiError::Internal("Failed to finish room".to_string()))?;

    if updated.is_ranked {
        if let Some(tid) = updated.tournament_id.clone() {
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
        }
    }
    Ok(updated)
}
