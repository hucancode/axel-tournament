use anyhow::Result;
use bollard::Docker;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

mod api_client;
mod docker_runner;

use api_client::ApiClient;
use docker_runner::DockerRunner;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://api:8080".to_string());
    let api_key = std::env::var("MATCH_RUNNER_API_KEY")
        .expect("MATCH_RUNNER_API_KEY must be set");

    info!("Judge service starting...");
    info!("API URL: {}", api_url);

    let api_client = ApiClient::new(&api_url, &api_key);
    let docker = Docker::connect_with_socket_defaults()?;
    let docker_runner = DockerRunner::new(docker);

    info!("Judge service ready. Polling for matches...");

    // Main loop: poll for pending matches
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        match api_client.fetch_pending_matches().await {
            Ok(matches) => {
                if !matches.is_empty() {
                    info!("Found {} pending match(es)", matches.len());
                }

                for match_data in matches {
                    info!("Processing match: {}", match_data.id);

                    // Mark match as queued
                    if let Err(e) = api_client.update_match_status(&match_data.id, "queued").await
                    {
                        error!("Failed to update match status to queued: {}", e);
                        continue;
                    }

                    // Mark match as running
                    if let Err(e) = api_client.update_match_status(&match_data.id, "running").await
                    {
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
                            error!("Match execution failed: {}", e);
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
            Err(e) => {
                error!("Failed to fetch pending matches: {}", e);
            }
        }
    }
}
