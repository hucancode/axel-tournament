use crate::{
    error::ApiResult,
    models::{LeaderboardEntry, LeaderboardQuery},
    services,
    AppState,
};
use axum::{extract::{Query, State}, Json};

pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> ApiResult<Json<Vec<LeaderboardEntry>>> {
    let limit = query.limit.unwrap_or(100);

    let entries = services::leaderboard::get_leaderboard(
        &state.db,
        limit,
        query.tournament_id.as_deref(),
        query.game_id.as_deref(),
    )
    .await?;

    Ok(Json(entries))
}
