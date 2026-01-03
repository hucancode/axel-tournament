mod app_state;
mod capacity;
mod compiler;
mod config;
mod db;
mod game_logic;
mod game_metadata;
mod games;
mod match_watcher;
mod room;
mod player;
mod players;
mod websocket_handler;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use app_state::AppState;

use capacity::CapacityTracker;
use config::Config;
use game_logic::GameLogic;
use room::RoomManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    dotenv::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Starting game server on port {}", config.server_port);
    tracing::info!("Max capacity: {}, Max claim delay: {}ms", config.max_capacity, config.max_claim_delay_ms);

    // Connect to database
    let db = db::connect(
        &config.database_url,
        &config.database_ns,
        &config.database_db,
        &config.database_user,
        &config.database_pass,
    ).await?;

    // Initialize capacity tracker
    let capacity = CapacityTracker::new(config.max_capacity, config.max_claim_delay_ms);

    // Start match watchers for automated games (AI vs AI)
    let db_clone = db.clone();
    let capacity_clone = capacity.clone();
    tokio::spawn(async move {
        if let Err(e) = match_watcher::start_match_watcher(
            db_clone.clone(),
            games::TicTacToe,
            "tic-tac-toe".to_string(),
            capacity_clone.clone(),
        ).await {
            tracing::error!("Tic-Tac-Toe match watcher error: {}", e);
        }
    });

    let db_clone = db.clone();
    let capacity_clone = capacity.clone();
    tokio::spawn(async move {
        if let Err(e) = match_watcher::start_match_watcher(
            db_clone.clone(),
            games::RockPaperScissors,
            "rock-paper-scissors".to_string(),
            capacity_clone.clone(),
        ).await {
            tracing::error!("Rock-Paper-Scissors match watcher error: {}", e);
        }
    });

    let db_clone = db.clone();
    let capacity_clone = capacity.clone();
    tokio::spawn(async move {
        if let Err(e) = match_watcher::start_match_watcher(
            db_clone.clone(),
            games::PrisonersDilemma,
            "prisoners-dilemma".to_string(),
            capacity_clone.clone(),
        ).await {
            tracing::error!("Prisoner's Dilemma match watcher error: {}", e);
        }
    });

    // Start room watchers for human games (WebSocket players)
    // Create app state for each game type
    let tic_tac_toe_state = Arc::new(AppState {
        db: db.clone(),
        game: games::TicTacToe::new(),
        capacity: capacity.clone(),
        room_manager: RoomManager::new(),
    });

    let rps_state = Arc::new(AppState {
        db: db.clone(),
        game: games::RockPaperScissors::new(),
        capacity: capacity.clone(),
        room_manager: RoomManager::new(),
    });

    let pd_state = Arc::new(AppState {
        db: db.clone(),
        game: games::PrisonersDilemma::new(),
        capacity: capacity.clone(),
        room_manager: RoomManager::new(),
    });

    // Build router with game-specific WebSocket endpoints
    let app = Router::new()
        .route("/health", get(health))
        .route("/capacity", get(get_capacity))
        // Room management API
        .route("/api/rooms", post(room::create_room::<games::TicTacToe>))
        .route("/api/rooms/:room_id", get(room::get_room::<games::TicTacToe>))
        .route("/api/rooms/:room_id/join", post(room::join_room::<games::TicTacToe>))
        .route("/api/rooms/:room_id/start", post(room::start_game::<games::TicTacToe>))
        .with_state(tic_tac_toe_state.clone())
        // WebSocket endpoints
        .route("/room/tic-tac-toe/:room_id", get(websocket_handler::websocket_room_handler::<games::TicTacToe>))
        .with_state(tic_tac_toe_state.clone())
        .route("/room/rock-paper-scissors/:room_id", get(websocket_handler::websocket_room_handler::<games::RockPaperScissors>))
        .with_state(rps_state.clone())
        .route("/room/prisoners-dilemma/:room_id", get(websocket_handler::websocket_room_handler::<games::PrisonersDilemma>))
        .with_state(pd_state.clone())
        .layer(CorsLayer::permissive());

    // Start server
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Game server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn get_capacity(
    axum::extract::State(state): axum::extract::State<Arc<AppState<games::TicTacToe>>>,
) -> axum::Json<capacity::CapacityStats> {
    let stats = state.capacity.get_stats().await;
    axum::Json(stats)
}
