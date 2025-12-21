use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Datetime, Thing};
use surrealdb::Surreal;
use tracing::{error, info, warn};

const STATUS_PENDING: &str = "pending";
const STATUS_RUNNING: &str = "running";

#[derive(Clone, Debug)]
struct HealerConfig {
    db_url: String,
    db_user: String,
    db_pass: String,
    db_ns: String,
    db_name: String,
    pending_stale_seconds: i64,
    running_stale_seconds: i64,
    sweep_interval_seconds: u64,
}

impl HealerConfig {
    fn from_env() -> Self {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "ws://surrealdb:8000".to_string());
        let db_user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string());
        let db_ns = std::env::var("DATABASE_NS").unwrap_or_else(|_| "axel".to_string());
        let db_name = std::env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string());
        let pending_stale_seconds = std::env::var("HEALER_PENDING_STALE_SECONDS")
            .unwrap_or_else(|_| "120".to_string())
            .parse()
            .unwrap_or(120);
        let running_stale_seconds = std::env::var("HEALER_RUNNING_STALE_SECONDS")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .unwrap_or(600);
        let sweep_interval_seconds = std::env::var("HEALER_SWEEP_INTERVAL_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Self {
            db_url,
            db_user,
            db_pass,
            db_ns,
            db_name,
            pending_stale_seconds,
            running_stale_seconds,
            sweep_interval_seconds,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MatchRow {
    id: Thing,
    status: String,
    #[serde(default)]
    updated_at: Option<Datetime>,
}

#[derive(Debug, Clone)]
struct MatchState {
    status: String,
    updated_at: Option<DateTime<Utc>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = HealerConfig::from_env();
    run_healer(config).await
}

async fn run_healer(config: HealerConfig) -> Result<()> {
    let endpoint = config.db_url.trim_start_matches("ws://");
    let db: Surreal<Client> = Surreal::new::<Ws>(endpoint).await?;

    db.signin(Root {
        username: &config.db_user,
        password: &config.db_pass,
    })
    .await?;
    db.use_ns(&config.db_ns).use_db(&config.db_name).await?;
    info!("Healer connected to SurrealDB at {}", config.db_url);

    let mut tracked: HashMap<String, MatchState> = HashMap::new();
    loop {
        tracked.clear();
        if let Err(err) = seed_matches(&db, &mut tracked).await {
            error!("Healer failed to seed matches: {}", err);
        }

        let response = db
            .query("LIVE SELECT id, status, updated_at FROM match WHERE status IN ['pending','running']")
            .await;
        let mut response = match response {
            Ok(response) => response,
            Err(err) => {
                error!("Healer live query setup failed: {}", err);
                tokio::time::sleep(StdDuration::from_secs(5)).await;
                continue;
            }
        };
        let mut stream = match response.stream::<surrealdb::Notification<MatchRow>>(0) {
            Ok(stream) => stream,
            Err(err) => {
                error!("Healer live query stream creation failed: {}", err);
                tokio::time::sleep(StdDuration::from_secs(5)).await;
                continue;
            }
        };

        let mut interval =
            tokio::time::interval(StdDuration::from_secs(config.sweep_interval_seconds));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = heal_stale(&db, &mut tracked, &config).await {
                        error!("Healer sweep failed: {}", err);
                    }
                }
                next = stream.next() => {
                    match next {
                        Some(Ok(notification)) => {
                            handle_notification(&mut tracked, notification);
                        }
                        Some(Err(err)) => {
                            error!("Healer live query error: {}", err);
                        }
                        None => {
                            warn!("Healer live query stream ended, resubscribing...");
                            break;
                        }
                    }
                }
            }
        }

        tokio::time::sleep(StdDuration::from_secs(2)).await;
    }
}

async fn seed_matches(db: &Surreal<Client>, tracked: &mut HashMap<String, MatchState>) -> Result<()> {
    let mut response = db
        .query("SELECT id, status, updated_at FROM match WHERE status IN ['pending','running']")
        .await?;
    let matches: Vec<MatchRow> = response.take(0)?;
    for row in matches {
        tracked.insert(
            row.id.to_string(),
            MatchState {
                status: row.status,
                updated_at: row.updated_at.map(Into::into),
            },
        );
    }
    Ok(())
}

fn handle_notification(
    tracked: &mut HashMap<String, MatchState>,
    notification: surrealdb::Notification<MatchRow>,
) {
    let row = notification.data;
    let state = MatchState {
        status: row.status,
        updated_at: row.updated_at.map(Into::into),
    };
    match notification.action {
        surrealdb::Action::Delete => {
            tracked.remove(&row.id.to_string());
        }
        _ => {
            tracked.insert(row.id.to_string(), state);
        }
    }
}

async fn heal_stale(
    db: &Surreal<Client>,
    tracked: &mut HashMap<String, MatchState>,
    config: &HealerConfig,
) -> Result<()> {
    let now = Utc::now();
    let mut to_remove = Vec::new();

    for (match_id, state) in tracked.iter_mut() {
        let limit = match state.status.as_str() {
            STATUS_PENDING => config.pending_stale_seconds,
            STATUS_RUNNING => config.running_stale_seconds,
            _ => {
                to_remove.push(match_id.clone());
                continue;
            }
        };
        if limit <= 0 {
            continue;
        }

        let is_stale = match state.updated_at {
            Some(updated_at) => now - updated_at > Duration::seconds(limit),
            None => true,
        };
        if !is_stale {
            continue;
        }

        let updated = if state.status == STATUS_PENDING {
            touch_pending(db, match_id).await?
        } else {
            requeue_running(db, match_id).await?
        };

        if updated {
            state.status = STATUS_PENDING.to_string();
            state.updated_at = Some(now);
        } else {
            to_remove.push(match_id.clone());
        }
    }

    for match_id in to_remove {
        tracked.remove(&match_id);
    }

    Ok(())
}

async fn touch_pending(db: &Surreal<Client>, match_id: &str) -> Result<bool> {
    let match_thing: Thing = match_id
        .parse()
        .map_err(|_| anyhow!("Invalid match id {}", match_id))?;
    let mut response = db
        .query(
            "UPDATE $match_id SET updated_at = time::now()
             WHERE status = 'pending' RETURN AFTER",
        )
        .bind(("match_id", match_thing))
        .await?;
    let updated: Vec<MatchRow> = response.take(0)?;
    Ok(!updated.is_empty())
}

async fn requeue_running(db: &Surreal<Client>, match_id: &str) -> Result<bool> {
    let match_thing: Thing = match_id
        .parse()
        .map_err(|_| anyhow!("Invalid match id {}", match_id))?;
    let mut response = db
        .query(
            "UPDATE $match_id SET status = 'pending', started_at = NONE, updated_at = time::now()
             WHERE status = 'running' RETURN AFTER",
        )
        .bind(("match_id", match_thing))
        .await?;
    let updated: Vec<MatchRow> = response.take(0)?;
    Ok(!updated.is_empty())
}
