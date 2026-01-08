use crate::app_state::AppState;
use crate::capacity::CapacityStats;
use crate::games::Game;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

pub async fn health() -> &'static str {
    "OK"
}

pub async fn get_capacity<G: Game>(
    State(state): State<Arc<AppState<G>>>,
) -> Json<CapacityStats> {
    let stats = state.capacity.get_stats().await;
    Json(stats)
}
