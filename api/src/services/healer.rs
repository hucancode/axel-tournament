use crate::db::Database;
use crate::models::tournament::Tournament;
use crate::services::bracket::advance_brackets;
use crate::services::finalization::finalize_if_done;
use std::time::Duration;
use tracing::{error, info};

pub struct HealerService {
    db: Database,
}

impl HealerService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn run(self) {
        info!("Healer service started");
        loop {
            tick(&self.db).await;
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}

/// One healer pass. Public so integration tests can drive it without
/// waiting on the 30s timer.
pub async fn tick(db: &Database) {
    if let Err(e) = db
        .query(
            "UPDATE match SET status = 'pending', updated_at = time::now()
             WHERE status = 'queued' AND updated_at < time::now() - 5m",
        )
        .await
    {
        error!("refresh stale matches: {e}");
    }

    if let Err(e) = db
        .query(
            "UPDATE match SET status = 'pending', updated_at = time::now()
             WHERE status = 'running' AND started_at < time::now() - 10m",
        )
        .await
    {
        error!("re-queue stale running matches: {e}");
    }

    if let Err(e) = db
        .query("DELETE FROM room WHERE updated_at < time::now() - 30m")
        .await
    {
        error!("clean up stale rooms: {e}");
    }

    if let Err(e) = db
        .query("DELETE FROM match WHERE room_id != NONE AND room_id NOT IN (SELECT id FROM room)")
        .await
    {
        error!("clean up orphaned matches: {e}");
    }

    // Scheduled tournaments whose start_time has arrived flip into
    // Registration so players can join.
    if let Err(e) = db
        .query(
            "UPDATE tournament SET status = 'registration', updated_at = time::now()
             WHERE status = 'scheduled' AND start_time != NONE AND start_time <= time::now()",
        )
        .await
    {
        error!("scheduled -> registration: {e}");
    }

    // Aggregate match scores into participants for every running
    // tournament; finalize when no non-terminal matches remain.
    let running_resp = db
        .query("SELECT * FROM tournament WHERE status IN ['running', 'generating']")
        .await;
    match running_resp {
        Ok(mut resp) => {
            let running: Vec<Tournament> = resp.take(0).unwrap_or_default();
            for t in running {
                if let Some(tid) = t.id {
                    if let Err(e) = advance_brackets(db, tid.clone()).await {
                        error!("advance_brackets: {e}");
                    }
                    if let Err(e) = finalize_if_done(db, tid).await {
                        error!("finalize_if_done: {e}");
                    }
                }
            }
        }
        Err(e) => error!("query running tournaments: {e}"),
    }
}
