use anyhow::Result;
use bollard::Docker;
use futures_util::StreamExt;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::{Surreal, opt::auth::Root};
use tracing::{error, info};

mod api_client;
mod docker_runner;

use api_client::ApiClient;
use docker_runner::DockerRunner;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "ws://surrealdb:8000".to_string());
    let db_user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string());
    let db_ns = std::env::var("DATABASE_NS").unwrap_or_else(|_| "tournament".to_string());
    let db_name = std::env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string());

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://api:8080".to_string());
    let api_key = std::env::var("MATCH_RUNNER_API_KEY")
        .expect("MATCH_RUNNER_API_KEY must be set");

    info!("Judge service starting...");
    info!("Database URL: {}", db_url);
    info!("API URL: {}", api_url);

    // Connect to SurrealDB
    let endpoint = db_url.trim_start_matches("ws://");
    let db: Surreal<Client> = Surreal::new::<Ws>(endpoint).await?;

    db.signin(Root {
        username: &db_user,
        password: &db_pass,
    }).await?;

    db.use_ns(&db_ns).use_db(&db_name).await?;
    info!("Connected to SurrealDB");

    let api_client = ApiClient::new(&api_url, &api_key);
    let docker = Docker::connect_with_socket_defaults()?;
    let docker_runner = DockerRunner::new(docker, api_client.clone());

    // Build universal Docker image on startup
    info!("Ensuring universal Docker image is built...");
    if let Err(e) = docker_runner.ensure_universal_image().await {
        error!("Failed to build universal Docker image: {}", e);
        error!("Judge service cannot start without the universal image");
        return Err(e);
    }

    info!("Judge service ready. Subscribing to pending matches...");

    // Subscribe to pending matches using LIVE query
    let mut response = db
        .query("LIVE SELECT * FROM match WHERE status = 'pending'")
        .await?;

    let mut stream = response.stream::<surrealdb::Notification<serde_json::Value>>(0)?;

    info!("Listening for pending matches via LIVE query...");

    while let Some(result) = stream.next().await {
        match result {
            Ok(notification) => {
                let action = notification.action;

                if action == surrealdb::Action::Create || action == surrealdb::Action::Update {
                    // Parse match data from notification result
                    if let Ok(match_data) = serde_json::from_value::<api_client::Match>(notification.data) {
                        // Only process if status is pending (double check since LIVE query filters)
                        if match_data.status == "pending" {
                            info!("Received pending match: {}", match_data.id);

                            // Mark match as queued
                            if let Err(e) = api_client.update_match_status(&match_data.id, "queued").await {
                                error!("Failed to update match status to queued: {}", e);
                                continue;
                            }

                            // Mark match as running
                            if let Err(e) = api_client.update_match_status(&match_data.id, "running").await {
                                error!("Failed to update match status to running: {}", e);
                                continue;
                            }

                            // Execute match
                            match docker_runner.execute_match(&match_data).await {
                                Ok(result) => {
                                    info!(
                                        "Match {} completed successfully. Reporting results...",
                                        match_data.id
                                    );
                                    // Report results
                                    if let Err(e) =
                                        api_client.report_match_result(&match_data.id, result).await
                                    {
                                        error!("Failed to report match result: {}", e);
                                    } else {
                                        info!("Match {} results reported successfully", match_data.id);
                                    }
                                }
                                Err(e) => {
                                    error!("Match {} execution failed: {}", match_data.id, e);
                                    // Report failure
                                    if let Err(e) = api_client
                                        .report_match_failure(&match_data.id, &e.to_string())
                                        .await
                                    {
                                        error!("Failed to report match failure: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("LIVE query error: {}", e);
                // Could reconnect here
            }
        }
    }

    Ok(())
}
