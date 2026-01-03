use anyhow::Result;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;

pub type Database = Surreal<Client>;

pub async fn connect(
    url: &str,
    namespace: &str,
    database: &str,
    user: &str,
    pass: &str,
) -> Result<Database> {
    let max_retries = 10;
    let mut retry_count = 0;

    loop {
        match Surreal::new::<Ws>(url).await {
            Ok(db) => {
                match db.signin(Root {
                    username: user,
                    password: pass,
                }).await {
                    Ok(_) => {
                        db.use_ns(namespace).use_db(database).await?;
                        tracing::info!("Successfully connected to database at {}", url);
                        return Ok(db);
                    }
                    Err(e) if retry_count < max_retries => {
                        retry_count += 1;
                        tracing::warn!(
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
                tracing::warn!(
                    "Database connection failed (attempt {}/{}): {}. Retrying in 2s...",
                    retry_count, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
