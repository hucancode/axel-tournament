mod app_state;
mod auth;
mod capacity;
mod compiler;
mod config;
mod db;
mod games;
mod match_watcher;
mod room;
mod players;
mod sandbox;

use anyhow::Result;
use axum::{
    http::{Method, header},
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use app_state::AppState;

use capacity::CapacityTracker;
use config::Config;
use games::Game;
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
            games::TicTacToe::new(),
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
            games::RockPaperScissors::new(),
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
            games::PrisonersDilemma::new(),
            "prisoners-dilemma".to_string(),
            capacity_clone.clone(),
        ).await {
            tracing::error!("Prisoner's Dilemma match watcher error: {}", e);
        }
    });

    // Create a shared RoomManager for all game types
    let shared_room_manager = Arc::new(RoomManager::new());

    // Create app state for each game type (sharing the same RoomManager)
    let tic_tac_toe_state = Arc::new(AppState {
        db: db.clone(),
        game: games::TicTacToe::new(),
        capacity: capacity.clone(),
        room_manager: shared_room_manager.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });

    let rps_state = Arc::new(AppState {
        db: db.clone(),
        game: games::RockPaperScissors::new(),
        capacity: capacity.clone(),
        room_manager: shared_room_manager.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });

    let pd_state = Arc::new(AppState {
        db: db.clone(),
        game: games::PrisonersDilemma::new(),
        capacity: capacity.clone(),
        room_manager: shared_room_manager.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });

    // Build router with game-specific WebSocket endpoints
    let app = Router::new()
        .route("/health", get(health))
        .route("/capacity", get(get_capacity))
        // Room management API (use tic_tac_toe_state but RoomManager is shared)
        .route("/api/rooms", get(room::list_rooms::<games::TicTacToe>).post(room::create_room::<games::TicTacToe>))
        .route("/api/rooms/{room_id}", get(room::get_room::<games::TicTacToe>))
        .route("/api/rooms/{room_id}/join", post(room::join_room::<games::TicTacToe>))
        .route("/api/rooms/{room_id}/leave", delete(room::leave_room::<games::TicTacToe>))
        .route("/api/rooms/{room_id}/start", post(room::start_game::<games::TicTacToe>))
        .with_state(tic_tac_toe_state.clone())
        // WebSocket endpoints
        .route("/ws/tic-tac-toe/{room_id}", get(room::ws_get_room::<games::TicTacToe>))
        .with_state(tic_tac_toe_state.clone())
        .route("/ws/rock-paper-scissors/{room_id}", get(room::ws_get_room::<games::RockPaperScissors>))
        .with_state(rps_state.clone())
        .route("/ws/prisoners-dilemma/{room_id}", get(room::ws_get_room::<games::PrisonersDilemma>))
        .with_state(pd_state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(
                    std::env::var("FRONTEND_URL")
                        .unwrap_or_else(|_| "http://localhost:5173".to_string())
                        .parse::<axum::http::HeaderValue>()
                        .expect("Invalid FRONTEND_URL"),
                )
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                ])
                .allow_credentials(true)
        );

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting judge server on {}", addr);
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
