use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use tokio::time::{sleep, Duration};

use crate::capacity::CapacityTracker;
use crate::compiler::Compiler;
use crate::games::{Game, GameResult};
use crate::players::Player;
use crate::players::BotPlayer;

type Database = Surreal<Client>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Match {
    id: Thing,
    tournament_id: Thing,
    game_id: String,
    status: String,
    participants: Vec<MatchParticipant>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
    #[serde(default)]
    room_id: Option<Thing>,
    created_at: Datetime,
    updated_at: Datetime,
    #[serde(default)]
    started_at: Option<Datetime>,
    #[serde(default)]
    completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchParticipant {
    #[serde(default)]
    user_id: Option<Thing>,
    submission_id: Thing,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Submission {
    #[serde(default)]
    id: Option<Thing>,
    #[serde(default)]
    compiled_binary_path: Option<String>,
    language: String,
    code: String,
}

/// Watches for pending automated matches and executes them
pub async fn start_match_watcher<G>(
    db: Database,
    game: G,
    game_id: String,
    capacity: CapacityTracker,
) -> Result<()>
where
    G: Game + Clone + Send + Sync + 'static,
{
    tracing::info!("Starting match watcher for game: {}", game_id);

    loop {
        // Query for pending matches
        let query = format!(
            "SELECT * FROM match WHERE game_id = '{}' AND status = 'pending' AND tournament_id != NONE ORDER BY created_at LIMIT 10",
            game_id
        );

        let mut response = db.query(&query).await.context("Failed to execute match query")?;
        let matches: Vec<Match> = response.take(0).context("Failed to deserialize matches")?;

        for match_record in matches {
            let match_id_str = match_record.id.to_string();

            // Check if we can accept more work
            if !capacity.can_accept_work().await {
                tracing::debug!("At capacity, skipping match claiming");
                continue;
            }

            // Calculate delay based on current load
            let delay_ms = capacity.calculate_claim_delay().await;
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }

            // Try to claim the match atomically by updating status pending -> queued
            let claim_query = "UPDATE $match_id SET status = 'queued', updated_at = time::now() WHERE status = 'pending' RETURN AFTER";

            let mut claim_result_query = db.query(claim_query)
                .bind(("match_id", match_record.id.clone()))
                .await.context("Failed to execute claim query")?;
            let claim_result: Vec<Match> = claim_result_query.take(0).context("Failed to deserialize claim results")?;

            if claim_result.is_empty() {
                tracing::debug!("Match {} already claimed by another server", match_id_str);
                continue;
            }

            tracing::info!("Claimed match: {}", match_id_str);
            capacity.increment_matches().await;

            // Spawn task to execute match
            let db_clone = db.clone();
            let capacity_clone = capacity.clone();
            let game_clone = game.clone();
            let match_id_clone = match_id_str.clone();

            tokio::spawn(async move {
                let result = execute_match(db_clone.clone(), game_clone, match_record).await;

                match result {
                    Ok(_) => {
                        tracing::info!("Match {} completed successfully", match_id_clone);
                    }
                    Err(e) => {
                        tracing::error!("Match {} failed: {}", match_id_clone, e);
                        // Update match status to failed
                        let _ = db_clone
                            .query("UPDATE $match_id SET status = 'failed', error_message = $error, updated_at = time::now(), completed_at = time::now()")
                            .bind(("match_id", match_id_clone.clone()))
                            .bind(("error", e.to_string()))
                            .await;
                    }
                }

                capacity_clone.decrement_matches().await;
            });
        }

        // Sleep before next poll
        sleep(Duration::from_secs(2)).await;
    }
}

async fn execute_match<G>(db: Database, game: G, match_record: Match) -> Result<()>
where
    G: Game,
{
    let match_id_str = match_record.id.to_string();

    // Update status to running
    db.query("UPDATE $match_id SET status = 'running', started_at = time::now(), updated_at = time::now()")
        .bind(("match_id", match_record.id.clone()))
        .await?;

    // Get game metadata for timeouts
    let game_metadata = crate::games::find_game_by_id(&match_record.game_id)
        .ok_or_else(|| anyhow::anyhow!("Game metadata not found for {}", match_record.game_id))?;

    // Initialize compiler
    let compiler = Compiler::new()?;

    // Get submission binaries from participants
    let mut binary_paths = Vec::new();

    for participant in &match_record.participants {
        let submission_id_str = participant.submission_id.to_string();

        let mut result = db
            .query("SELECT compiled_binary_path, language, code FROM $submission_id")
            .bind(("submission_id", participant.submission_id.clone()))
            .await
            .context(format!("Failed to query submission {}", submission_id_str))?;

        let submissions: Vec<Submission> = result.take(0)
            .context(format!("Failed to deserialize submission {}", submission_id_str))?;

        if let Some(submission) = submissions.first() {
            let binary_path = match &submission.compiled_binary_path {
                Some(path) => path.clone(),
                None => {
                    // Submission not compiled yet, compile it now
                    tracing::info!("Compiling submission {} on-demand", submission_id_str);

                    let compiled_path = compiler
                        .compile_submission(&submission_id_str, &submission.language, &submission.code)
                        .await
                        .context(format!("Failed to compile submission {}", submission_id_str))?;

                    // Update submission with compiled binary path
                    db.query("UPDATE $submission_id SET compiled_binary_path = $binary_path, status = 'accepted'")
                        .bind(("submission_id", participant.submission_id.clone()))
                        .bind(("binary_path", compiled_path.clone()))
                        .await
                        .context(format!("Failed to update submission {} with binary path", submission_id_str))?;

                    compiled_path
                }
            };
            binary_paths.push(binary_path);
        } else {
            anyhow::bail!("Submission {} not found", submission_id_str);
        }
    }

    if binary_paths.len() != 2 {
        anyhow::bail!("Expected 2 binaries, got {}", binary_paths.len());
    }

    tracing::info!("Executing match {} with binaries: {:?}", match_id_str, binary_paths);

    let mut player1 = BotPlayer::new("p1".to_string(), &binary_paths[0]).await?;
    let mut player2 = BotPlayer::new("p2".to_string(), &binary_paths[1]).await?;

    // Set bot timeouts
    player1.set_timeout(game_metadata.bot_turn_timeout_ms);
    player2.set_timeout(game_metadata.bot_turn_timeout_ms);

    let players: Vec<Box<dyn crate::players::Player>> = vec![Box::new(player1), Box::new(player2)];

    // Execute the game
    let results = game.run(players, game_metadata.bot_turn_timeout_ms).await;

    // Convert GameResult enum to scores
    let score0 = match results[0] {
        GameResult::Accepted(score) => score as f64,
        _ => 0.0,
    };
    let score1 = match results[1] {
        GameResult::Accepted(score) => score as f64,
        _ => 0.0,
    };

    db.query(
        "UPDATE $match_id SET
            status = 'completed',
            participants[0].score = $score0,
            participants[1].score = $score1,
            completed_at = time::now(),
            updated_at = time::now()"
    )
    .bind(("match_id", match_record.id))
    .bind(("score0", score0))
    .bind(("score1", score1))
    .await?;

    tracing::info!("Match {} completed with scores: {} {}", match_id_str, score0, score1);

    Ok(())
}
