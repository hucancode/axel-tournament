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
                    username: user.to_string(),
                    password: pass.to_string(),
                }).await {
                    Ok(_) => {
                        db.use_ns(namespace).use_db(database).await?;
                        init_schema(&db).await?;
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

/// Idempotently ensure tables judge queries exist. API later layers SCHEMAFULL
/// fields onto `match`/`submission`. Judge-owned tables (`room_*`) stay schemaless.
async fn init_schema(db: &Database) -> Result<()> {
    db.query(
        "DEFINE TABLE IF NOT EXISTS match SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS tournament_id ON match TYPE option<record<tournament>>;
         DEFINE FIELD IF NOT EXISTS game_id ON match TYPE string;
         DEFINE FIELD IF NOT EXISTS room_id ON match TYPE option<record<room>>;
         DEFINE FIELD IF NOT EXISTS status ON match TYPE string;
         DEFINE FIELD IF NOT EXISTS participants ON match TYPE array<{
             user_id: record<user>,
             submission_id: option<record<submission>>,
             score: option<float>
         }>;
         DEFINE FIELD IF NOT EXISTS metadata ON match TYPE option<object>;
         DEFINE FIELD IF NOT EXISTS error_message ON match TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS faulted_user_ids ON match TYPE array<record<user>> DEFAULT [];
         DEFINE FIELD IF NOT EXISTS round ON match TYPE option<number>;
         DEFINE FIELD IF NOT EXISTS bracket ON match TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS bracket_position ON match TYPE option<number>;
         DEFINE FIELD IF NOT EXISTS game_event_source ON match TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS judge_server_name ON match TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS created_at ON match TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON match TYPE datetime;
         DEFINE FIELD IF NOT EXISTS started_at ON match TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS completed_at ON match TYPE option<datetime>;
         DEFINE TABLE IF NOT EXISTS submission SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS user_id ON submission TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS tournament_id ON submission TYPE record<tournament>;
         DEFINE FIELD IF NOT EXISTS game_id ON submission TYPE string;
         DEFINE FIELD IF NOT EXISTS language ON submission TYPE string;
         DEFINE FIELD IF NOT EXISTS code ON submission TYPE string;
         DEFINE FIELD IF NOT EXISTS status ON submission TYPE string DEFAULT 'pending';
         DEFINE FIELD IF NOT EXISTS error_message ON submission TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS compiled_binary_path ON submission TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS created_at ON submission TYPE datetime;
         DEFINE TABLE IF NOT EXISTS room_lease SCHEMALESS;
         DEFINE TABLE IF NOT EXISTS room_event SCHEMALESS;
         DEFINE TABLE IF NOT EXISTS room_meta SCHEMALESS;
         DEFINE INDEX IF NOT EXISTS room_event_seq ON room_event FIELDS room, seq UNIQUE;",
    )
    .await?;
    Ok(())
}
