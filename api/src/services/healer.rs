use crate::db::{Database, SeedCreds};
use crate::models::matches::Match;
use crate::models::tournament::Tournament;
use crate::services::bracket::advance_brackets;
use crate::services::elo;
use crate::services::finalization::finalize_if_done;
use crate::services::matchmaking;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

const HEALER_INTERVAL: Duration = Duration::from_secs(30);
const BOOTSTRAP_INTERVAL: Duration = Duration::from_secs(10);

/// Background loop. Runs heavyweight `tick` every `HEALER_INTERVAL`.
/// Schema + seed restoration runs on a separate fast loop (see
/// `run_bootstrap_loop`) so dropped tables come back in seconds, not
/// minutes.
pub async fn run_healer(db: Database) {
    info!("Healer service started");
    loop {
        tick(&db).await;
        tokio::time::sleep(HEALER_INTERVAL).await;
    }
}

/// Fast loop that keeps the database schema + dev users present. Re-runs
/// idempotent migrations and re-seeds the three dev users whenever the
/// user table is empty. Short interval so an operator dropping a table
/// at runtime doesn't lock out the API for long.
pub async fn run_bootstrap_loop(db: Database, seed: Option<Arc<SeedCreds>>) {
    info!("Bootstrap loop started");
    loop {
        if let Err(e) = bootstrap(&db, seed.as_deref()).await {
            error!("bootstrap: {e}");
        }
        tokio::time::sleep(BOOTSTRAP_INTERVAL).await;
    }
}

/// Re-apply idempotent schema and (if seed creds provided) re-seed dev
/// users when the user table is empty. Safe to call repeatedly.
pub async fn bootstrap(
    db: &Database,
    creds: Option<&SeedCreds>,
) -> Result<(), surrealdb::Error> {
    // Cheap probe: if the `user` table is queryable, schema is intact and
    // we skip re-running the full DDL set. Recovery only pays the DDL cost
    // when an operator has actually dropped a table.
    let schema_intact = db
        .query("SELECT VALUE id FROM user LIMIT 1")
        .await
        .and_then(|mut r| r.take::<Vec<surrealdb::types::RecordId>>(0))
        .is_ok();
    if !schema_intact {
        if let Err(e) = axel_core::migrations::ensure_schema(db).await {
            error!("ensure_schema: {e}");
        }
    }
    if let Some(c) = creds {
        crate::db::seed_users(db, c).await?;
    }
    Ok(())
}

/// One healer pass. Public so integration tests can drive it without
/// waiting on the timer.
pub async fn tick(db: &Database) {
    // Independent timeout/orphan sweeps — fan out in parallel.
    let (a, b, c, d, e) = tokio::join!(
        refresh_stale_queued(db),
        requeue_stale_running(db),
        cleanup_stale_rooms(db),
        cleanup_orphan_matches(db),
        promote_scheduled_tournaments(db),
    );
    if let Err(e) = a {
        error!("refresh stale queued: {e}");
    }
    if let Err(e) = b {
        error!("requeue stale running: {e}");
    }
    if let Err(e) = c {
        error!("cleanup stale rooms: {e}");
    }
    if let Err(e) = d {
        error!("cleanup orphan matches: {e}");
    }
    if let Err(e) = e {
        error!("scheduled -> registration: {e}");
    }
    if let Err(e) = apply_pending_elo(db).await {
        error!("apply pending elo: {e}");
    }
    if let Err(e) = requeue_continuous_players(db).await {
        error!("requeue continuous players: {e}");
    }
    if let Err(e) = run_matchmaker_for_continuous(db).await {
        error!("matchmaker tick: {e}");
    }
    if let Err(e) = advance_running_tournaments(db).await {
        error!("advance running tournaments: {e}");
    }
}

/// Run the matchmaker pairing pass for every running Continuous
/// tournament. Pairs queued players (closest score) into ranked rooms.
async fn run_matchmaker_for_continuous(db: &Database) -> Result<(), surrealdb::Error> {
    let mut resp = db
        .query(
            "SELECT * FROM tournament
             WHERE status = 'running'
               AND match_generation_type = 'continuous'",
        )
        .await?;
    let running: Vec<Tournament> = resp.take(0).unwrap_or_default();
    for t in running {
        let Some(tid) = t.id else { continue };
        if let Err(e) = matchmaking::tick(db, tid).await {
            error!("matchmaker: {e}");
        }
    }
    Ok(())
}

/// For each running `Continuous` tournament, re-enqueue any joined
/// participant who isn't currently queued and isn't currently in a
/// non-terminal ranked room. Keeps the score-pairing matchmaker fed
/// without players manually re-clicking "queue" between hands.
async fn requeue_continuous_players(db: &Database) -> Result<(), surrealdb::Error> {
    let mut resp = db
        .query(
            "SELECT * FROM tournament
             WHERE status = 'running'
               AND match_generation_type = 'continuous'",
        )
        .await?;
    let running: Vec<Tournament> = resp.take(0).unwrap_or_default();
    for t in running {
        let Some(tid) = t.id.clone() else { continue };
        // Past end_time? Skip — finalization will close it.
        if let Some(et) = t.end_time.as_ref() {
            let now = surrealdb::types::Datetime::default();
            if et.to_string() <= now.to_string() {
                continue;
            }
        }
        if let Err(e) = db
            .query(
                "LET $busy = (SELECT VALUE players FROM room
                              WHERE tournament_id = $tid
                                AND status IN ['lobby', 'playing']);
                 LET $queued = (SELECT VALUE user_id FROM matchmaking_ticket
                                WHERE tournament_id = $tid);
                 LET $busy_set = array::flatten($busy);
                 FOR $p IN (SELECT VALUE user_id FROM tournament_participant
                            WHERE tournament_id = $tid) {
                     IF !($p IN $queued) AND !($p IN $busy_set) {
                         CREATE matchmaking_ticket SET
                             tournament_id = $tid,
                             user_id = $p,
                             elo = (SELECT VALUE elo FROM tournament_participant
                                    WHERE tournament_id = $tid
                                      AND user_id = $p
                                    LIMIT 1)[0] ?? 1000.0,
                             queued_at = time::now();
                     }
                 };",
            )
            .bind(("tid", tid))
            .await
        {
            error!("requeue: {e}");
        }
    }
    Ok(())
}

/// Process every ranked terminal match that hasn't yet had its ELO /
/// score adjustment applied. Idempotent — guarded by `match.elo_applied`.
/// The judge writes match rows for ranked human rooms via
/// `match_writer::db_finish_callback`; this pass picks them up and
/// updates the rating column.
async fn apply_pending_elo(db: &Database) -> Result<(), surrealdb::Error> {
    let mut resp = db
        .query(
            "SELECT * FROM match
             WHERE elo_applied = false
               AND status IN ['completed', 'failed']
               AND tournament_id != NONE
               AND room_id != NONE
               AND room_id.is_ranked = true
             LIMIT 200",
        )
        .await?;
    let rows: Vec<Match> = resp.take(0).unwrap_or_default();
    for m in rows {
        let Some(mid) = m.id.clone() else { continue };
        let Some(tid) = m.tournament_id.clone() else { continue };
        let players: Vec<_> = m.participants.iter().map(|p| p.user_id.clone()).collect();
        let winner = m.winner().map(|p| p.user_id);
        let aligned: Vec<f64> = m
            .participants
            .iter()
            .map(|p| p.score.unwrap_or(0.0))
            .collect();
        if let Err(e) = elo::apply_ranked_result(
            db,
            &tid,
            &m.game_id,
            &players,
            &winner,
            Some(aligned.as_slice()),
        )
        .await
        {
            error!("apply_ranked_result for {mid:?}: {e}");
            continue;
        }
        if let Err(e) = db
            .query("UPDATE $mid SET elo_applied = true")
            .bind(("mid", mid))
            .await
        {
            error!("mark elo_applied: {e}");
        }
    }
    Ok(())
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
