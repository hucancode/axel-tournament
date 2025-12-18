use anyhow::Result;
use axel_tournament_judge::{run_judge, JudgeConfig};
use tracing::warn;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    let config = JudgeConfig::from_env();
    let shutdown = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("Failed to listen for shutdown signal: {}", e);
        }
    };
    run_judge(config, shutdown).await
}
