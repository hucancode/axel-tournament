use crate::capacity::CapacityTracker;
use crate::db::Database;
use crate::games::{self, Game};
use crate::match_watcher;
use crate::room::RoomManager;
use std::sync::Arc;

/// Start all match watchers for automated games (AI vs AI)
pub fn start_match_watchers(db: Database, capacity: CapacityTracker) {
    // Tic-Tac-Toe watcher
    let db_clone = db.clone();
    let capacity_clone = capacity.clone();
    tokio::spawn(async move {
        if let Err(e) = match_watcher::start_match_watcher(
            db_clone.clone(),
            games::TicTacToe::new(),
            "tic-tac-toe".to_string(),
            capacity_clone.clone(),
        )
        .await
        {
            tracing::error!("Tic-Tac-Toe match watcher error: {}", e);
        }
    });

    // Rock-Paper-Scissors watcher
    let db_clone = db.clone();
    let capacity_clone = capacity.clone();
    tokio::spawn(async move {
        if let Err(e) = match_watcher::start_match_watcher(
            db_clone.clone(),
            games::RockPaperScissors::new(),
            "rock-paper-scissors".to_string(),
            capacity_clone.clone(),
        )
        .await
        {
            tracing::error!("Rock-Paper-Scissors match watcher error: {}", e);
        }
    });

    // Prisoner's Dilemma watcher
    let db_clone = db.clone();
    let capacity_clone = capacity.clone();
    tokio::spawn(async move {
        if let Err(e) = match_watcher::start_match_watcher(
            db_clone.clone(),
            games::PrisonersDilemma::new(),
            "prisoners-dilemma".to_string(),
            capacity_clone.clone(),
        )
        .await
        {
            tracing::error!("Prisoner's Dilemma match watcher error: {}", e);
        }
    });
}

/// Recover orphaned rooms from previous server crashes
pub async fn recover_orphaned_rooms(room_manager: &Arc<RoomManager>) {
    if let Err(e) = room_manager.recover_orphaned_rooms().await {
        tracing::error!("Failed to recover orphaned rooms: {}", e);
    }
}
