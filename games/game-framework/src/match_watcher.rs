use crate::automated_executor::execute_automated_match;
use crate::db_client::{DbClient, MatchParticipant, MatchRow, ParticipantResult};
use crate::game_trait::{GameConfig, GameLogic};
use anyhow::Result;
use futures_util::StreamExt;
use std::time::Duration as StdDuration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{error, info, warn};

pub struct MatchWatcherConfig {
    pub game_id: String,
    pub db_url: String,
    pub db_user: String,
    pub db_pass: String,
    pub db_ns: String,
    pub db_name: String,
    pub game_config: GameConfig,
}

pub async fn run_match_watcher<G: GameLogic>(config: MatchWatcherConfig) -> Result<()> {
    let endpoint = config.db_url.trim_start_matches("ws://");
    let db: Surreal<Client> = Surreal::new::<Ws>(endpoint).await?;

    db.signin(Root {
        username: &config.db_user,
        password: &config.db_pass,
    })
    .await?;
    db.use_ns(&config.db_ns).use_db(&config.db_name).await?;
    info!("Match watcher connected to SurrealDB at {}", config.db_url);

    loop {
        let response = db
            .query(&format!(
                "LIVE SELECT id, game_id, tournament_id, participants, status
                 FROM match
                 WHERE game_id = '{}' AND status = 'pending'",
                config.game_id
            ))
            .await;

        let mut response = match response {
            Ok(response) => response,
            Err(err) => {
                error!("Match watcher live query setup failed: {}", err);
                tokio::time::sleep(StdDuration::from_secs(5)).await;
                continue;
            }
        };

        let mut stream = match response.stream::<surrealdb::Notification<MatchRow>>(0) {
            Ok(stream) => stream,
            Err(err) => {
                error!("Match watcher live query stream creation failed: {}", err);
                tokio::time::sleep(StdDuration::from_secs(5)).await;
                continue;
            }
        };

        info!("Match watcher started for {}", config.game_id);

        loop {
            match stream.next().await {
                Some(Ok(notification)) => {
                    let match_row = notification.data;
                    info!("Received match: {:?}", match_row.id);

                    let db_client_clone = DbClient::new(db.clone());
                    let match_id = match_row.id.clone();
                    let participants = match_row.participants.clone();
                    let game_config = config.game_config.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_match::<G>(db_client_clone, match_id, participants, game_config)
                                .await
                        {
                            error!("Match handling failed: {}", e);
                        }
                    });
                }
                Some(Err(err)) => {
                    error!("Match watcher live query error: {}", err);
                }
                None => {
                    warn!("Match watcher live query stream ended, resubscribing...");
                    break;
                }
            }
        }

        tokio::time::sleep(StdDuration::from_secs(2)).await;
    }
}

async fn handle_match<G: GameLogic>(
    db_client: DbClient,
    match_id: surrealdb::sql::Thing,
    participants: Vec<MatchParticipant>,
    game_config: GameConfig,
) -> Result<()> {
    // Try to claim the match
    let claimed = db_client.try_claim_match(match_id.clone()).await?;
    if !claimed {
        info!("Match {:?} already claimed by another server", match_id);
        return Ok(());
    }

    info!("Executing match {:?}", match_id);
    db_client.update_match_running(match_id.clone()).await?;

    // Validate participant count
    if participants.len() != 2 {
        db_client
            .update_match_failed(
                match_id,
                format!("Invalid participant count: {}", participants.len()),
            )
            .await?;
        return Ok(());
    }

    // Fetch submissions and binary paths
    let sub1 = db_client
        .get_submission(participants[0].submission_id.clone())
        .await?;
    let sub2 = db_client
        .get_submission(participants[1].submission_id.clone())
        .await?;

    let binary1 = sub1
        .compiled_binary_path
        .ok_or_else(|| anyhow::anyhow!("Player 1 binary not found"))?;
    let binary2 = sub2
        .compiled_binary_path
        .ok_or_else(|| anyhow::anyhow!("Player 2 binary not found"))?;

    // Execute match using framework
    let result = execute_automated_match::<G>(&binary1, &binary2, &game_config).await;

    // Parse results
    let parts: Vec<&str> = result.trim().split_whitespace().collect();
    if parts.len() != 2 {
        db_client
            .update_match_failed(match_id, format!("Invalid result format: {}", result))
            .await?;
        return Ok(());
    }

    let (score1, error1) = match parts[0].parse::<i32>() {
        Ok(s) => (s, None),
        Err(_) => (0, Some(parts[0].to_string())),
    };

    let (score2, error2) = match parts[1].parse::<i32>() {
        Ok(s) => (s, None),
        Err(_) => (0, Some(parts[1].to_string())),
    };

    let results = vec![
        ParticipantResult {
            submission_id: participants[0].submission_id.clone(),
            user_id: participants[0].user_id.clone(),
            score: score1,
            error_code: error1,
        },
        ParticipantResult {
            submission_id: participants[1].submission_id.clone(),
            user_id: participants[1].user_id.clone(),
            score: score2,
            error_code: error2,
        },
    ];

    db_client.update_match_completed(match_id, results).await?;
    info!("Match completed successfully");

    Ok(())
}
