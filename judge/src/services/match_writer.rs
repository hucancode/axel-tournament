// Match writer: DB-side finalisation when a `turn_timer` watcher
// fires. Lifted out of `turn_timer` so the watcher stays pure (no DB,
// no business rules) and either piece can be unit-tested in isolation.
//
// Scoring on timeout:
//   - winner = the unique non-pending player, if exactly one survives
//   - faulted players score 0
//   - winner scores 1
//   - everyone else scores 0.5

use crate::db::Database;
use crate::services::match_finalizer::{FinishCallback, MatchOutcome};
use crate::services::turn_timer::TimeoutCallback;
use std::collections::HashSet;
use std::sync::Arc;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Build a `TimeoutCallback` that records a timeout match row and
/// finalises the room.
pub fn db_timeout_callback(db: Database) -> TimeoutCallback {
    Arc::new(move |room_id, pending| {
        let db = db.clone();
        Box::pin(async move {
            if let Err(e) = write_timeout_match(&db, &room_id, &pending).await {
                tracing::warn!("turn timeout writer failed for {room_id}: {e:#}");
            }
        })
    })
}

#[derive(serde::Deserialize, SurrealValue)]
struct RoomRow {
    players: Vec<RecordId>,
    #[serde(default)]
    tournament_id: Option<RecordId>,
    game_id: String,
    status: String,
}

#[derive(serde::Serialize, SurrealValue)]
struct Part {
    user_id: RecordId,
    submission_id: Option<RecordId>,
    score: Option<f64>,
}

async fn write_timeout_match(
    db: &Database,
    room_id: &str,
    pending: &[String],
) -> anyhow::Result<()> {
    let rid = parse_room_id(room_id);
    let mut resp = db
        .query("SELECT players, tournament_id, game_id, status FROM $rid")
        .bind(("rid", rid.clone()))
        .await?;
    let rows: Vec<RoomRow> = match resp.take(0) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("turn_timeout: room decode failed for {room_id}: {e}");
            return Ok(());
        }
    };
    let Some(room) = rows.into_iter().next() else {
        // Room not in api db (transient AI-only room).
        tracing::debug!("turn_timeout: no room found for {room_id}, skipping");
        return Ok(());
    };
    if room.status == "finished" {
        return Ok(());
    }

    let pending_set: HashSet<String> = pending.iter().cloned().collect();
    let is_pending = |p: &RecordId| pending_set.contains(&p.to_sql());
    let pending_records: Vec<RecordId> =
        room.players.iter().filter(|p| is_pending(p)).cloned().collect();
    let surviving: Vec<&RecordId> =
        room.players.iter().filter(|p| !is_pending(p)).collect();
    let winner = if surviving.len() == 1 {
        Some(surviving[0].clone())
    } else {
        None
    };

    let participants: Vec<Part> = room
        .players
        .iter()
        .map(|p| {
            let score = if Some(p) == winner.as_ref() {
                Some(1.0)
            } else if is_pending(p) {
                Some(0.0)
            } else {
                Some(0.5)
            };
            Part {
                user_id: p.clone(),
                submission_id: None,
                score,
            }
        })
        .collect();

    db.query(
        "CREATE match SET
             tournament_id = $tid,
             game_id = $gid,
             room_id = $rid,
             status = 'completed',
             participants = $parts,
             error_message = 'turn_timeout',
             faulted_user_ids = $faulted,
             created_at = time::now(),
             updated_at = time::now(),
             started_at = time::now(),
             completed_at = time::now();",
    )
    .bind(("tid", room.tournament_id.clone()))
    .bind(("gid", room.game_id.clone()))
    .bind(("rid", rid.clone()))
    .bind(("parts", participants))
    .bind(("faulted", pending_records))
    .await?;

    db.query(
        "UPDATE $rid SET status = 'finished',
                          winner_id = $winner,
                          updated_at = time::now()",
    )
    .bind(("rid", rid))
    .bind(("winner", winner))
    .await?;
    Ok(())
}

fn parse_room_id(s: &str) -> RecordId {
    RecordId::parse_simple(s).unwrap_or_else(|_| RecordId::new("room", s))
}

/// Build a `FinishCallback` that records a finished match row and
/// updates the room's status. Chip-aware: per-player scores from the
/// terminal event are written into `match.participants[].score` so
/// downstream ELO/score accumulation can use them.
pub fn db_finish_callback(db: Database) -> FinishCallback {
    Arc::new(move |room_id, outcome| {
        let db = db.clone();
        Box::pin(async move {
            if let Err(e) = write_finished_match(&db, &room_id, &outcome).await {
                tracing::warn!("match finish writer failed for {room_id}: {e:#}");
            }
        })
    })
}

async fn write_finished_match(
    db: &Database,
    room_id: &str,
    outcome: &MatchOutcome,
) -> anyhow::Result<()> {
    let rid = parse_room_id(room_id);
    let mut resp = db
        .query("SELECT players, tournament_id, game_id, status FROM $rid")
        .bind(("rid", rid.clone()))
        .await?;
    let rows: Vec<RoomRow> = match resp.take(0) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("match_finish: room decode failed for {room_id}: {e}");
            return Ok(());
        }
    };
    let Some(room) = rows.into_iter().next() else {
        // Room is not in api db (transient AI-only room).
        tracing::debug!("match_finish: no room found for {room_id}, skipping");
        return Ok(());
    };
    if room.status == "finished" {
        return Ok(());
    }
    if room.players.len() != outcome.scores.len() {
        tracing::warn!(
            "match_finish: player count mismatch for {room_id} ({} vs {})",
            room.players.len(),
            outcome.scores.len(),
        );
        return Ok(());
    }

    let winner = outcome
        .winner_idx
        .and_then(|i| room.players.get(i))
        .cloned();

    let participants: Vec<Part> = room
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| Part {
            user_id: p.clone(),
            submission_id: None,
            score: Some(outcome.scores[i]),
        })
        .collect();

    db.query(
        "CREATE match SET
             tournament_id = $tid,
             game_id = $gid,
             room_id = $rid,
             status = 'completed',
             participants = $parts,
             faulted_user_ids = [],
             created_at = time::now(),
             updated_at = time::now(),
             started_at = time::now(),
             completed_at = time::now();",
    )
    .bind(("tid", room.tournament_id.clone()))
    .bind(("gid", room.game_id.clone()))
    .bind(("rid", rid.clone()))
    .bind(("parts", participants))
    .await?;

    db.query(
        "UPDATE $rid SET status = 'finished',
                          winner_id = $winner,
                          updated_at = time::now()",
    )
    .bind(("rid", rid))
    .bind(("winner", winner))
    .await?;
    Ok(())
}
