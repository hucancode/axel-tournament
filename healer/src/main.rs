mod compiler;
mod db_client;

use anyhow::Result;
use compiler::Compiler;
use db_client::{DbClient, SubmissionRow};
use futures_util::StreamExt;
use std::time::Duration as StdDuration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{error, info, warn};

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
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "ws://surrealdb:8000".to_string());
        let db_user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string());
        let db_ns = std::env::var("DATABASE_NS").unwrap_or_else(|_| "axel".to_string());
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

    let db_client = DbClient::new(db.clone());
    let compiler = Compiler::new()?;

    // Ensure compiler images are pulled
    info!("Pulling compiler images...");
    compiler.ensure_images().await?;
    info!("Compiler images ready");

    loop {
        // Set up LIVE query for pending submissions
        let response = db
            .query("LIVE SELECT id, game_id, language, code, status FROM submission WHERE status = 'pending'")
            .await;

        let mut response = match response {
            Ok(response) => response,
            Err(err) => {
                error!("Healer live query setup failed: {}", err);
                tokio::time::sleep(StdDuration::from_secs(5)).await;
                continue;
            }
        };

        let mut stream = match response.stream::<surrealdb::Notification<SubmissionRow>>(0) {
            Ok(stream) => stream,
            Err(err) => {
                error!("Healer live query stream creation failed: {}", err);
                tokio::time::sleep(StdDuration::from_secs(5)).await;
                continue;
            }
        };

        info!("Healer compilation service started, watching for pending submissions");

        loop {
            match stream.next().await {
                Some(Ok(notification)) => {
                    let submission = notification.data;
                    info!("Received submission: {:?}", submission.id);

                    // Process compilation in background
                    let db_client_clone = DbClient::new(db.clone());
                    let compiler_clone = Compiler::new().unwrap();
                    let submission_id = submission.id.clone();
                    let language = submission.language.clone();
                    let code = submission.code.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_compilation(
                            db_client_clone,
                            compiler_clone,
                            submission_id,
                            language,
                            code,
                        )
                        .await
                        {
                            error!("Compilation handling failed: {}", e);
                        }
                    });
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

        tokio::time::sleep(StdDuration::from_secs(2)).await;
    }
}

async fn handle_compilation(
    db_client: DbClient,
    compiler: Compiler,
    submission_id: surrealdb::sql::Thing,
    language: String,
    code: String,
) -> Result<()> {
    // Try to claim the submission
    let claimed = db_client.try_claim_submission(submission_id.clone()).await?;
    if !claimed {
        info!("Submission {:?} already claimed by another healer", submission_id);
        return Ok(());
    }

    info!("Compiling submission {:?} ({})", submission_id, language);

    // Perform compilation
    let submission_id_str = submission_id.to_string();
    let result = compiler
        .compile_submission(&submission_id_str, &language, &code)
        .await;

    match result {
        Ok(binary_path) => {
            info!(
                "Compilation succeeded for {:?}: {}",
                submission_id, binary_path
            );
            db_client
                .update_compilation_success(submission_id, binary_path)
                .await?;
        }
        Err(e) => {
            warn!("Compilation failed for {:?}: {}", submission_id, e);
            db_client
                .update_compilation_failure(submission_id, e.to_string())
                .await?;
        }
    }

    Ok(())
}
