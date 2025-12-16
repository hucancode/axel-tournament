use crate::config::DatabaseConfig;
use crate::models::{User, UserRole};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Datetime;

// Type alias for database connection
pub type Database = Surreal<Client>;

pub async fn connect(config: &DatabaseConfig) -> Result<Database, surrealdb::Error> {
    // Remove ws:// prefix if present, as Surreal::new::<Ws> expects just host:port
    let endpoint = config.url.trim_start_matches("ws://");
    let db = Surreal::new::<Ws>(endpoint).await?;
    db.signin(Root {
        username: &config.user,
        password: &config.pass,
    })
    .await?;
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;
    // Initialize schema
    init_schema(&db).await?;
    Ok(db)
}

pub async fn init_schema(db: &Database) -> Result<(), surrealdb::Error> {
    // Define tables with proper constraints
    // Users table
    db.query(
        "DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS email ON user TYPE string ASSERT string::is::email($value);
         DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
         DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS role ON user TYPE string;
         DEFINE FIELD IF NOT EXISTS location ON user TYPE string;
         DEFINE FIELD IF NOT EXISTS oauth_provider ON user TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS oauth_id ON user TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS is_banned ON user TYPE bool DEFAULT false;
         DEFINE FIELD IF NOT EXISTS ban_reason ON user TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON user TYPE datetime;
         DEFINE FIELD IF NOT EXISTS password_reset_token ON user TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS password_reset_expires ON user TYPE option<datetime>;
         DEFINE INDEX IF NOT EXISTS unique_email ON user COLUMNS email UNIQUE;
         DEFINE INDEX IF NOT EXISTS unique_username ON user COLUMNS username UNIQUE;",
    )
    .await?;
    // Games table
    db.query(
        "DEFINE TABLE IF NOT EXISTS game SCHEMALESS;
         DEFINE FIELD IF NOT EXISTS name ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS description ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS is_active ON game TYPE bool DEFAULT true;
         DEFINE FIELD IF NOT EXISTS created_at ON game TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON game TYPE datetime;
         DEFINE INDEX IF NOT EXISTS unique_game_name ON game COLUMNS name UNIQUE;",
    )
    .await?;
    // Tournaments table
    db.query(
        "DEFINE TABLE IF NOT EXISTS tournament SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS game_id ON tournament TYPE record<game>;
         DEFINE FIELD IF NOT EXISTS name ON tournament TYPE string;
         DEFINE FIELD IF NOT EXISTS description ON tournament TYPE string;
         DEFINE FIELD IF NOT EXISTS status ON tournament TYPE string;
         DEFINE FIELD IF NOT EXISTS min_players ON tournament TYPE number;
         DEFINE FIELD IF NOT EXISTS max_players ON tournament TYPE number;
         DEFINE FIELD IF NOT EXISTS current_players ON tournament TYPE number DEFAULT 0;
         DEFINE FIELD IF NOT EXISTS start_time ON tournament TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS end_time ON tournament TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS match_generation_type ON tournament TYPE string DEFAULT 'all_vs_all';
         DEFINE FIELD IF NOT EXISTS matches_generated ON tournament TYPE bool DEFAULT false;
         DEFINE FIELD IF NOT EXISTS created_at ON tournament TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON tournament TYPE datetime;",
    )
    .await?;
    // Tournament participants table
    db.query(
        "DEFINE TABLE IF NOT EXISTS tournament_participant SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS tournament_id ON tournament_participant TYPE record<tournament>;
         DEFINE FIELD IF NOT EXISTS user_id ON tournament_participant TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS submission_id ON tournament_participant TYPE option<record<submission>>;
         DEFINE FIELD IF NOT EXISTS score ON tournament_participant TYPE number DEFAULT 0;
         DEFINE FIELD IF NOT EXISTS rank ON tournament_participant TYPE option<number>;
         DEFINE FIELD IF NOT EXISTS joined_at ON tournament_participant TYPE datetime;
         DEFINE INDEX IF NOT EXISTS unique_tournament_user ON tournament_participant COLUMNS tournament_id, user_id UNIQUE;"
    ).await?;
    // Submissions table
    db.query(
        "DEFINE TABLE IF NOT EXISTS submission SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS user_id ON submission TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS tournament_id ON submission TYPE record<tournament>;
         DEFINE FIELD IF NOT EXISTS game_id ON submission TYPE record<game>;
         DEFINE FIELD IF NOT EXISTS language ON submission TYPE string;
         DEFINE FIELD IF NOT EXISTS code ON submission TYPE string;
         DEFINE FIELD IF NOT EXISTS file_path ON submission TYPE string;
         DEFINE FIELD IF NOT EXISTS status ON submission TYPE string DEFAULT 'pending';
         DEFINE FIELD IF NOT EXISTS error_message ON submission TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS created_at ON submission TYPE datetime;",
    )
    .await?;
    // Matches table - SCHEMALESS to allow flexible participants array with nested objects
    db.query(
        "DEFINE TABLE IF NOT EXISTS match SCHEMALESS;
         DEFINE FIELD IF NOT EXISTS tournament_id ON match TYPE option<record<tournament>>;
         DEFINE FIELD IF NOT EXISTS game_id ON match TYPE record<game>;
         DEFINE FIELD IF NOT EXISTS status ON match TYPE string;
         DEFINE FIELD IF NOT EXISTS created_at ON match TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON match TYPE datetime;
         DEFINE INDEX IF NOT EXISTS idx_match_tournament ON match COLUMNS tournament_id;
         DEFINE INDEX IF NOT EXISTS idx_match_status ON match COLUMNS status;
         DEFINE INDEX IF NOT EXISTS idx_match_created ON match COLUMNS created_at;",
    )
    .await?;

    // Add indexes for performance on commonly queried fields
    db.query(
        "DEFINE INDEX IF NOT EXISTS idx_tournament_status ON tournament COLUMNS status;
         DEFINE INDEX IF NOT EXISTS idx_tournament_game ON tournament COLUMNS game_id;
         DEFINE INDEX IF NOT EXISTS idx_tournament_created ON tournament COLUMNS created_at;
         DEFINE INDEX IF NOT EXISTS idx_submission_user ON submission COLUMNS user_id;
         DEFINE INDEX IF NOT EXISTS idx_submission_tournament ON submission COLUMNS tournament_id;
         DEFINE INDEX IF NOT EXISTS idx_game_created ON game COLUMNS created_at;",
    )
    .await?;

    Ok(())
}

/// Create admin user if user table is empty (seed user)
pub async fn create_admin_user(
    db: &Database,
    email: &str,
    password_hash: String,
) -> Result<(), surrealdb::Error> {
    // Check if any users exist
    let existing: Vec<User> = db.query("SELECT * FROM user LIMIT 1").await?.take(0)?;
    if existing.is_empty() {
        // Create admin user (seed user only if table is empty)
        let admin = User {
            id: None,
            email: email.to_string(),
            username: "admin".to_string(),
            password_hash: Some(password_hash),
            role: UserRole::Admin,
            location: "US".to_string(),
            oauth_provider: None,
            oauth_id: None,
            is_banned: false,
            ban_reason: None,
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
            password_reset_token: None,
            password_reset_expires: None,
        };
        let _: Option<User> = db.create("user").content(admin).await?;
    }
    Ok(())
}
