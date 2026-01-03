use anyhow::Result;
use std::time::Duration as StdDuration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{error, info};

#[derive(Clone, Debug)]
struct HealerConfig {
    db_url: String,
    db_user: String,
    db_pass: String,
    db_ns: String,
    db_name: String,
}

impl HealerConfig {
    fn from_env() -> Self {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "localhost:8000".to_string());
        let db_user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string());
        let db_ns = std::env::var("DATABASE_NS").unwrap_or_else(|_| "tournament".to_string());
        let db_name = std::env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string());

        Self {
            db_url,
            db_user,
            db_pass,
            db_ns,
            db_name,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    let config = HealerConfig::from_env();
    run_healer(config).await
}

async fn run_healer(config: HealerConfig) -> Result<()> {
    let db: Surreal<Client> = Surreal::new::<Ws>(config.db_url.clone()).await?;

    db.signin(Root {
        username: &config.db_user,
        password: &config.db_pass,
    })
    .await?;
    db.use_ns(&config.db_ns).use_db(&config.db_name).await?;
    info!("Healer connected to SurrealDB at {}", config.db_url);

    loop {
        // Refresh stale pending matches (older than 5 minutes)
        let refresh_result = db
            .query("UPDATE match SET status = 'pending', updated_at = time::now() WHERE status = 'queued' AND updated_at < time::now() - 5m")
            .await;

        match refresh_result {
            Ok(_) => {
                info!("Refreshed stale queued matches back to pending");
            }
            Err(err) => {
                error!("Failed to refresh stale matches: {}", err);
            }
        }

        // Re-queue stale running matches (older than 10 minutes)
        let requeue_result = db
            .query("UPDATE match SET status = 'pending', updated_at = time::now() WHERE status = 'running' AND started_at < time::now() - 10m")
            .await;

        match requeue_result {
            Ok(_) => {
                info!("Re-queued stale running matches back to pending");
            }
            Err(err) => {
                error!("Failed to re-queue stale running matches: {}", err);
            }
        }

        // Sleep before next healing cycle
        tokio::time::sleep(StdDuration::from_secs(30)).await;
    }
}


