use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use tracing::{error, info, warn};

use crate::error::{AppError, AppResult};

pub type Database = Surreal<Client>;

#[derive(Debug, Clone)]
pub struct DbConnectConfig<'a> {
    pub url: &'a str,
    pub user: &'a str,
    pub pass: &'a str,
    pub namespace: &'a str,
    pub database: &'a str,
}

/// Connect to SurrealDB with bounded retry, sign in as root, select
/// ns/db, and apply the canonical migrations. Returns an `AppResult`
/// so both binaries share one error shape.
pub async fn connect(cfg: DbConnectConfig<'_>) -> AppResult<Database> {
    let max_retries = 10;
    let mut retry_count = 0;

    loop {
        match Surreal::new::<Ws>(cfg.url).await {
            Ok(db) => {
                match db
                    .signin(Root {
                        username: cfg.user.to_string(),
                        password: cfg.pass.to_string(),
                    })
                    .await
                {
                    Ok(_) => {
                        db.use_ns(cfg.namespace).use_db(cfg.database).await?;
                        crate::migrations::run(&db)
                            .await
                            .map_err(|e| AppError::Internal(format!("migrations: {e}")))?;
                        info!("Successfully connected to database at {}", cfg.url);
                        return Ok(db);
                    }
                    Err(e) if retry_count < max_retries => {
                        retry_count += 1;
                        warn!(
                            "Database signin failed (attempt {}/{}): {}. Retrying in 2s...",
                            retry_count, max_retries, e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                error!(
                    "Database connection failed (attempt {}/{}): {}. Retrying in 2s...",
                    retry_count, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
