use judge::{
    app_state::AppState,
    services::capacity::CapacityTracker,
    config::Config,
    db,
    games::{self, Game},
    services::room::RoomManager,
    router,
    services,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
    tracing::info!("Starting game server on port {}", config.server_port);
    tracing::info!(
        "Max capacity: {}, Max claim delay: {}ms",
        config.max_capacity,
        config.max_claim_delay_ms
    );
    // Connect to database
    let db = db::connect(
        &config.database_url,
        &config.database_ns,
        &config.database_db,
        &config.database_user,
        &config.database_pass,
    )
    .await?;
    // Initialize capacity tracker
    let capacity = CapacityTracker::new(config.max_capacity, config.max_claim_delay_ms);

    // Start match watchers for automated games (AI vs AI)
    services::start_match_watchers(db.clone(), capacity.clone());

    // Create a shared RoomManager for all game types
    let shared_room_manager = Arc::new(RoomManager::new(db.clone()));

    // Recover orphaned rooms from previous server crashes
    services::recover_orphaned_rooms(&shared_room_manager).await;

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
    // Create router
    let app = router::create_router(&config, tic_tac_toe_state, rps_state, pd_state);
    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting judge server on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
