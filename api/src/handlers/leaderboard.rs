use crate::{
    AppState,
    error::ApiResult,
    models::{LeaderboardEntry, LeaderboardQuery, rid},
    services,
};
use axum::{
    Json,
    extract::{Query, State},
};

pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> ApiResult<Json<Vec<LeaderboardEntry>>> {
    let limit = query.limit.unwrap_or(100);
    let tournament_id = query.tournament_id.map(|id| rid("tournament", id));
    let game_id = query.game_id.map(|id| rid("game", id));
    let entries =
        services::leaderboard::get_leaderboard(&state.db, limit, tournament_id, game_id).await?;
    Ok(Json(entries))
}
