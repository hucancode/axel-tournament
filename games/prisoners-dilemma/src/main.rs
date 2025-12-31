mod game_logic;

use anyhow::Result;
use game_framework::{run_http_server, run_match_watcher, GameConfig, GameMetadata, GameType, MatchWatcherConfig, ProgrammingLanguage};
use game_logic::PrisonersDilemmaGame;
use tracing::error;

const GAME_ID: &str = "prisoners-dilemma";
const SERVER_PORT: u16 = 8083;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let metadata = GameMetadata {
        id: GAME_ID,
        name: "Prisoner's Dilemma",
        description: "Classic game theory prisoner's dilemma for 2 players - supports both automated and interactive modes",
        game_type: GameType::Automated,
        supported_languages: vec![
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        server_port: SERVER_PORT,
        rounds_per_match: 100,
        repetitions: 1,
        timeout_ms: 5000,
        cpu_limit: 1.0,
        turn_timeout_ms: 2000,
        memory_limit_mb: 64,
    };

    let http_server = tokio::spawn(async move {
        if let Err(e) = run_http_server::<PrisonersDilemmaGame>(SERVER_PORT, metadata).await {
            error!("HTTP server error: {}", e);
        }
    });

    let match_watcher = tokio::spawn(async move {
        let config = MatchWatcherConfig {
            game_id: GAME_ID.to_string(),
            db_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "ws://surrealdb:8000".to_string()),
            db_user: std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string()),
            db_pass: std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string()),
            db_ns: std::env::var("DATABASE_NS").unwrap_or_else(|_| "axel".to_string()),
            db_name: std::env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string()),
            game_config: GameConfig {
                num_rounds: 100,
                turn_timeout_ms: 2000,
                memory_limit_mb: 64,
                cpu_limit: 0.5,
            },
        };

        if let Err(e) = run_match_watcher::<PrisonersDilemmaGame>(config).await {
            error!("Match watcher error: {}", e);
        }
    });

    tokio::select! {
        _ = http_server => {},
        _ = match_watcher => {},
    }

    Ok(())
}
