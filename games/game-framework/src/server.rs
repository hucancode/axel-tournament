use crate::game_trait::GameLogic;
use crate::websocket_room::{websocket_handler, AppState};
use crate::GameMetadata;
use anyhow::Result;
use axum::{routing::get, Json, Router};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Run HTTP server for a game
/// Provides /health, /metadata, and /room/:room_id endpoints
pub async fn run_http_server<G: GameLogic>(
    port: u16,
    metadata: GameMetadata,
) -> Result<()> {
    let app_state: AppState<G> = Arc::new(RwLock::new(HashMap::new()));
    let metadata = Arc::new(metadata);

    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/metadata",
            get({
                let metadata = metadata.clone();
                move || async move { get_metadata(metadata).await }
            }),
        )
        .route("/room/:room_id", get(websocket_handler::<G>))
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Game server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_metadata(metadata: Arc<GameMetadata>) -> Json<GameMetadata> {
    Json((*metadata).clone())
}
