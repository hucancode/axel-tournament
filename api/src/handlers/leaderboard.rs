use crate::{
    AppState,
    error::AppResult,
    models::{LeaderboardEntry, LeaderboardQuery, rid},
    services,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};

pub async fn get_leaderboard(
    State(state): State<AppState>,
    Path(tournament_id): Path<String>,
    Query(query): Query<LeaderboardQuery>,
) -> AppResult<Json<Vec<LeaderboardEntry>>> {
    let limit = query.limit.unwrap_or(100);
    let tournament_id = rid("tournament", tournament_id);
    let entries = services::leaderboard::get_leaderboard(&state.db, tournament_id, limit).await?;
    Ok(Json(entries))
}
