use crate::db::Database;
use crate::models::tournament::Tournament;
use crate::services::bracket::advance_brackets;
use crate::services::finalization::finalize_if_done;
use std::time::Duration;
use tracing::{error, info};

const HEALER_INTERVAL: Duration = Duration::from_secs(30);

/// Background loop. Runs `tick` every `HEALER_INTERVAL`.
pub async fn run_healer(db: Database) {
    info!("Healer service started");
    loop {
        tick(&db).await;
        tokio::time::sleep(HEALER_INTERVAL).await;
    }
}

/// One healer pass. Public so integration tests can drive it without
/// waiting on the timer.
pub async fn tick(db: &Database) {
    if let Err(e) = refresh_stale_queued(db).await {
        error!("refresh stale queued: {e}");
    }
    if let Err(e) = requeue_stale_running(db).await {
        error!("requeue stale running: {e}");
    }
    if let Err(e) = cleanup_stale_rooms(db).await {
        error!("cleanup stale rooms: {e}");
    }
    if let Err(e) = cleanup_orphan_matches(db).await {
        error!("cleanup orphan matches: {e}");
    }
    if let Err(e) = promote_scheduled_tournaments(db).await {
        error!("scheduled -> registration: {e}");
    }
    if let Err(e) = advance_running_tournaments(db).await {
        error!("advance running tournaments: {e}");
    }
}

/// Match queued > 5m: re-bump to pending so a free judge picks it up.
async fn refresh_stale_queued(db: &Database) -> Result<(), surrealdb::Error> {
    db.query(
        "UPDATE match SET status = 'pending', updated_at = time::now()
         WHERE status = 'queued' AND updated_at < time::now() - 5m",
    )
    .await
    .map(|_| ())
}

/// Match running > 10m: judge died mid-game. Re-queue.
async fn requeue_stale_running(db: &Database) -> Result<(), surrealdb::Error> {
    db.query(
        "UPDATE match SET status = 'pending', updated_at = time::now()
         WHERE status = 'running' AND started_at < time::now() - 10m",
    )
    .await
    .map(|_| ())
}

/// Drop rooms idle > 30m.
async fn cleanup_stale_rooms(db: &Database) -> Result<(), surrealdb::Error> {
    db.query("DELETE FROM room WHERE updated_at < time::now() - 30m")
        .await
        .map(|_| ())
}

/// Match rows that point at a deleted room.
async fn cleanup_orphan_matches(db: &Database) -> Result<(), surrealdb::Error> {
    db.query("DELETE FROM match WHERE room_id != NONE AND room_id NOT IN (SELECT id FROM room)")
        .await
        .map(|_| ())
}

/// Scheduled tournaments whose start_time has arrived flip into Registration.
async fn promote_scheduled_tournaments(db: &Database) -> Result<(), surrealdb::Error> {
    db.query(
        "UPDATE tournament SET status = 'registration', updated_at = time::now()
         WHERE status = 'scheduled' AND start_time != NONE AND start_time <= time::now()",
    )
    .await
    .map(|_| ())
}

/// Aggregate match scores into participants for every running tournament;
/// finalize when no non-terminal matches remain.
async fn advance_running_tournaments(db: &Database) -> Result<(), surrealdb::Error> {
    let mut resp = db
        .query("SELECT * FROM tournament WHERE status IN ['running', 'generating']")
        .await?;
    let running: Vec<Tournament> = resp.take(0).unwrap_or_default();
    for t in running {
        let Some(tid) = t.id else { continue };
        if let Err(e) = advance_brackets(db, tid.clone()).await {
            error!("advance_brackets: {e}");
        }
        if let Err(e) = finalize_if_done(db, tid).await {
            error!("finalize_if_done: {e}");
        }
    }
    Ok(())
}
