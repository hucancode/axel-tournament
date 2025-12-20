use anyhow::{anyhow, Result};
use bollard::Docker;
use futures_util::StreamExt;
use serde::Deserialize;
use std::future::Future;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{error, info};

pub mod db_client;
pub mod docker_runner;

use db_client::{DbClient, Match, MatchParticipant};
use docker_runner::DockerRunner;

#[derive(Clone, Debug)]
pub struct JudgeConfig {
    pub db_url: String,
    pub db_user: String,
    pub db_pass: String,
    pub db_ns: String,
    pub db_name: String,
    pub sandbox_image: String,
}

impl JudgeConfig {
    pub fn from_env() -> Self {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "ws://surrealdb:8000".to_string());
        let db_user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string());
        let db_ns = std::env::var("DATABASE_NS").unwrap_or_else(|_| "axel".to_string());
        let db_name = std::env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string());
        let sandbox_image = std::env::var("JUDGE_SANDBOX_IMAGE").unwrap_or_else(|_| "axel-sandbox".to_string());

        Self {
            db_url,
            db_user,
            db_pass,
            db_ns,
            db_name,
            sandbox_image,
        }
    }
}

pub struct JudgeHandle {
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<()>>>,
}

impl JudgeHandle {
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            match task.await {
                Ok(res) => res,
                Err(e) => Err(anyhow!("Judge task join error: {}", e)),
            }
        } else {
            Ok(())
        }
    }
}

impl Drop for JudgeHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

pub fn spawn_judge_server(config: JudgeConfig) -> JudgeHandle {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        run_judge(config, async move {
            let _ = shutdown_rx.await;
        })
        .await
    });
    JudgeHandle {
        shutdown: Some(shutdown_tx),
        task: Some(task),
    }
}

pub async fn run_judge<S>(config: JudgeConfig, shutdown: S) -> Result<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    let endpoint = config.db_url.trim_start_matches("ws://");
    let db: Surreal<Client> = Surreal::new::<Ws>(endpoint).await?;

    db.signin(Root {
        username: &config.db_user,
        password: &config.db_pass,
    })
    .await?;

    db.use_ns(&config.db_ns).use_db(&config.db_name).await?;
    info!("Connected to SurrealDB at {}", config.db_url);

    let db = Arc::new(db);
    let db_client = DbClient::new(db.clone());
    let docker = Docker::connect_with_socket_defaults()?;
    let docker_runner =
        DockerRunner::new(docker, db_client.clone(), config.sandbox_image.clone());

    info!("Using sandbox image: {}", config.sandbox_image);
    docker_runner.ensure_image_present().await?;

    info!("Judge service ready. Subscribing to pending matches...");
    let mut response = db
        .query("LIVE SELECT * FROM match WHERE status = 'pending'")
        .await?;

    #[derive(Debug, Deserialize)]
    struct MatchNotification {
        id: Thing,
        game_id: Thing,
        #[serde(default)]
        tournament_id: Option<Thing>,
        status: String,
        participants: Vec<MatchNotificationParticipant>,
    }

    #[derive(Debug, Deserialize)]
    struct MatchNotificationParticipant {
        submission_id: Thing,
        #[serde(default)]
        score: Option<f64>,
        #[serde(default)]
        metadata: Option<serde_json::Value>,
    }

    let mut stream = response.stream::<surrealdb::Notification<MatchNotification>>(0)?;
    info!("Listening for match notifications via LIVE query...");
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            _ = &mut shutdown => {
                info!("Shutdown signal received, stopping judge loop");
                break;
            }
            next = stream.next() => {
                let Some(result) = next else {
                    info!("LIVE query stream ended");
                    break;
                };
                match result {
                    Ok(notification) => {
                        info!("Notification action: {:?}", notification.action);
                        let action = notification.action;
                        if action == surrealdb::Action::Create || action == surrealdb::Action::Update {
                            let live_match = notification.data;
                            if live_match.status != "pending" {
                                continue;
                            }
                            let match_id = live_match.id.to_string();
                            let participants: Vec<MatchParticipant> = live_match
                                .participants
                                .into_iter()
                                .map(|p| MatchParticipant {
                                    submission_id: p.submission_id,
                                    score: p.score,
                                    metadata: p.metadata,
                                })
                                .collect();

                            let match_data = Match {
                                id: match_id.clone(),
                                game_id: live_match.game_id.to_string(),
                                tournament_id: live_match.tournament_id.map(|t| t.to_string()),
                                status: live_match.status,
                                participants,
                            };

                            let claimed = match db_client.try_queue_match(&match_data.id).await {
                                Ok(claimed) => claimed,
                                Err(e) => {
                                    error!("Failed to update match status to queued: {}", e);
                                    continue;
                                }
                            };

                            if !claimed {
                                info!(
                                    "Match {} already queued by another judge, skipping",
                                    match_data.id
                                );
                                continue;
                            }

                            info!("Processing match: {}", match_data.id);

                            if let Err(e) = db_client.update_match_status(&match_data.id, "running").await {
                                error!("Failed to update match status to running: {}", e);
                                continue;
                            }

                            match docker_runner.execute_match(&match_data).await {
                                Ok(result) => {
                                    info!(
                                        "Match {} completed successfully. Reporting results...",
                                        match_data.id
                                    );
                                    if let Err(e) = db_client.report_match_result(&match_data, result).await {
                                        error!("Failed to report match result: {}", e);
                                    } else {
                                        info!("Match {} results reported successfully", match_data.id);
                                    }
                                }
                                Err(e) => {
                                    error!("Match {} execution failed: {}", match_data.id, e);
                                    let msg = e.to_string();
                                    let report_err = if msg.contains("GAME_CODE_COMPILATION_FAILED") {
                                        db_client.report_match_error(&match_data.id, &msg).await
                                    } else {
                                        db_client.report_match_failure(&match_data.id, &msg).await
                                    };
                                    if let Err(e) = report_err {
                                        error!("Failed to report match failure: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("LIVE query error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
