use crate::db::Database;
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
            // Refresh stale queued matches (queued for more than 5 minutes)
            let refresh_result = self
                .db
                .query("UPDATE match SET status = 'pending', updated_at = time::now() WHERE status = 'queued' AND updated_at < time::now() - 5m")
                .await;
            match refresh_result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to refresh stale matches: {}", err);
                }
            }

            // Re-queue stuck running matches (running for more than 10 minutes)
            let requeue_result = self
                .db
                .query("UPDATE match SET status = 'pending', updated_at = time::now() WHERE status = 'running' AND started_at < time::now() - 10m")
                .await;
            match requeue_result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to re-queue stale running matches: {}", err);
                }
            }

            // Clean up stale rooms (no activity for more than 30 minutes)
            let cleanup_rooms_result = self
                .db
                .query("DELETE FROM room WHERE updated_at < time::now() - 30m")
                .await;
            match cleanup_rooms_result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to clean up stale rooms: {}", err);
                }
            }

            // Clean up orphaned matches (matches linked to deleted rooms)
            let cleanup_matches_result = self
                .db
                .query("DELETE FROM match WHERE room_id != NONE AND room_id NOT IN (SELECT id FROM room)")
                .await;
            match cleanup_matches_result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to clean up orphaned matches: {}", err);
                }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}
