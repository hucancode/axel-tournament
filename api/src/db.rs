use crate::config::DatabaseConfig;
use crate::models::{User, UserRole};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Datetime, Thing};

// Type alias for database connection
pub type Database = Surreal<Client>;

pub async fn connect(config: &DatabaseConfig) -> Result<Database, surrealdb::Error> {
    // Remove ws:// prefix if present, as Surreal::new::<Ws> expects just host:port
    let endpoint = config.url.trim_start_matches("ws://");

    let max_retries = 10;
    let mut retry_count = 0;

    loop {
        match Surreal::new::<Ws>(endpoint).await {
            Ok(db) => {
                match db
                    .signin(Root {
                        username: &config.user,
                        password: &config.pass,
                    })
                    .await
                {
                    Ok(_) => {
                        db.use_ns(&config.namespace)
                            .use_db(&config.database)
                            .await?;
                        // Initialize schema
                        init_schema(&db).await?;
                        eprintln!("Successfully connected to database at {}", config.url);
                        return Ok(db);
                    }
                    Err(e) if retry_count < max_retries => {
                        retry_count += 1;
                        eprintln!(
                            "Database signin failed (attempt {}/{}): {}. Retrying in 2s...",
                            retry_count, max_retries, e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                eprintln!(
                    "Database connection failed (attempt {}/{}): {}. Retrying in 2s...",
                    retry_count, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e),
        }
    }
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
        "DEFINE TABLE IF NOT EXISTS game SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS name ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS description ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS supported_languages ON game TYPE array<string>;
         DEFINE FIELD IF NOT EXISTS is_active ON game TYPE bool DEFAULT true;
         DEFINE FIELD IF NOT EXISTS owner_id ON game TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS game_code ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS game_language ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS game_type ON game TYPE string;
         DEFINE FIELD IF NOT EXISTS frontend_code ON game TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS rounds_per_match ON game TYPE number;
         DEFINE FIELD IF NOT EXISTS repetitions ON game TYPE number;
         DEFINE FIELD IF NOT EXISTS timeout_ms ON game TYPE number;
         DEFINE FIELD IF NOT EXISTS cpu_limit ON game TYPE number;
         DEFINE FIELD IF NOT EXISTS turn_timeout_ms ON game TYPE number DEFAULT 2000;
         DEFINE FIELD IF NOT EXISTS memory_limit_mb ON game TYPE number DEFAULT 64;
         DEFINE FIELD IF NOT EXISTS created_at ON game TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON game TYPE datetime;
         DEFINE INDEX IF NOT EXISTS unique_game_name ON game COLUMNS name UNIQUE;
         DEFINE INDEX IF NOT EXISTS idx_game_owner ON game COLUMNS owner_id;",
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
         DEFINE FIELD IF NOT EXISTS start_time ON tournament TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS end_time ON tournament TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS match_generation_type ON tournament TYPE string DEFAULT 'all_vs_all';
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
         DEFINE FIELD IF NOT EXISTS status ON submission TYPE string DEFAULT 'pending';
         DEFINE FIELD IF NOT EXISTS error_message ON submission TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS created_at ON submission TYPE datetime;",
    )
    .await?;
    // Rooms table
    db.query(
        "DEFINE TABLE IF NOT EXISTS room SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS game_id ON room TYPE record<game>;
         DEFINE FIELD IF NOT EXISTS status ON room TYPE string;
         DEFINE FIELD IF NOT EXISTS participants ON room TYPE array;
         DEFINE FIELD IF NOT EXISTS created_at ON room TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON room TYPE datetime;
         DEFINE INDEX IF NOT EXISTS idx_room_game ON room COLUMNS game_id;
         DEFINE INDEX IF NOT EXISTS idx_room_status ON room COLUMNS status;",
    )
    .await?;

    // Matches table
    db.query(
        "DEFINE TABLE IF NOT EXISTS match SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS tournament_id ON match TYPE option<record<tournament>>;
         DEFINE FIELD IF NOT EXISTS game_id ON match TYPE record<game>;
         DEFINE FIELD IF NOT EXISTS room_id ON match TYPE option<record<room>>;
         DEFINE FIELD IF NOT EXISTS status ON match TYPE string;
         DEFINE FIELD IF NOT EXISTS participants ON match TYPE array;
         DEFINE FIELD IF NOT EXISTS metadata ON match TYPE option<object>;
         DEFINE FIELD IF NOT EXISTS created_at ON match TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON match TYPE datetime;
         DEFINE FIELD IF NOT EXISTS started_at ON match TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS completed_at ON match TYPE option<datetime>;
         DEFINE INDEX IF NOT EXISTS idx_match_tournament ON match COLUMNS tournament_id;
         DEFINE INDEX IF NOT EXISTS idx_match_room ON match COLUMNS room_id;
         DEFINE INDEX IF NOT EXISTS idx_match_status ON match COLUMNS status;
         DEFINE INDEX IF NOT EXISTS idx_match_created ON match COLUMNS created_at;",
    )
    .await?;

    // Game templates table
    db.query(
        "DEFINE TABLE IF NOT EXISTS game_template SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS game_id ON game_template TYPE record<game>;
         DEFINE FIELD IF NOT EXISTS language ON game_template TYPE string;
         DEFINE FIELD IF NOT EXISTS template_code ON game_template TYPE string;
         DEFINE FIELD IF NOT EXISTS created_at ON game_template TYPE datetime;
         DEFINE FIELD IF NOT EXISTS updated_at ON game_template TYPE datetime;
         DEFINE INDEX IF NOT EXISTS unique_game_language ON game_template COLUMNS game_id, language UNIQUE;",
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
pub async fn seed_admin_user(
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
        let admin_user: Option<User> = db.create("user").content(admin).await?;

        // Create initial games if admin user was created
        if let Some(admin) = admin_user {
            if let Err(e) = seed_initial_games(db, &admin.id.unwrap()).await {
                eprintln!("Failed to create initial games: {:?}", e);
            }
        }
    }
    Ok(())
}

/// Create initial games for the admin user
async fn seed_initial_games(db: &Database, admin_id: &Thing) -> Result<(), Box<dyn std::error::Error>> {
    use crate::models::game::{ProgrammingLanguage, GameType};
    use crate::services::game;

    // Rock Paper Scissors game
    let rps_code = include_str!("../../games/rock_paper_scissor/server.rs");

    let _ = game::create_game(
        db,
        "Rock Paper Scissors".to_string(),
        "Classic rock-paper-scissors game. Return 'rock', 'paper', or 'scissors'.".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go, ProgrammingLanguage::C],
        admin_id.to_string(),
        rps_code.to_string(),
        ProgrammingLanguage::Rust,
        None,
        10,
        1,
        1000,
        0.5,
        100,
        64,
    ).await;

    // Prisoner's Dilemma game
    let pd_code = include_str!("../../games/prisoner_dilema/server.rs");

    let _ = game::create_game(
        db,
        "Prisoner's Dilemma".to_string(),
        "Classic prisoner's dilemma. Return 'cooperate' or 'defect'.".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go, ProgrammingLanguage::C],
        admin_id.to_string(),
        pd_code.to_string(),
        ProgrammingLanguage::Rust,
        None,
        20,
        1,
        1000,
        0.5,
        100,
        64,
    ).await;

    // Interactive Tic-Tac-Toe game
    let ttt_server_code = include_str!("../../games/tic_tac_toe/server.rs");
    let ttt_frontend_code = include_str!("../../games/tic_tac_toe/client.html");

    let _ = game::create_game(
        db,
        "Interactive Tic-Tac-Toe".to_string(),
        "Classic tic-tac-toe game played in real-time between two players.".to_string(),
        GameType::Interactive,
        vec![ProgrammingLanguage::Rust],
        admin_id.to_string(),
        ttt_server_code.to_string(),
        ProgrammingLanguage::Rust,
        Some(ttt_frontend_code.to_string()),
        1,
        1,
        5000,
        0.5,
        2000,
        64,
    ).await;

    Ok(())
}
