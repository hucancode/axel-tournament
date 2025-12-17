use crate::{
    AppState,
    error::ApiResult,
    models::{LeaderboardEntry, LeaderboardQuery},
    services,
};
use axum::{
    Json,
    extract::{Query, State},
};
use surrealdb::sql::Thing;

pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> ApiResult<Json<Vec<LeaderboardEntry>>> {
    let limit = query.limit.unwrap_or(100);
    let tournament_id = query
        .tournament_id
        .as_deref()
        .map(|id| {
            id.parse::<Thing>()
                .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))
        })
        .transpose()?;
    let game_id = query
        .game_id
        .as_deref()
        .map(|id| {
            id.parse::<Thing>()
                .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))
        })
        .transpose()?;
    let entries = services::leaderboard::get_leaderboard(
        &state.db,
        limit,
        tournament_id,
        game_id,
    )
    .await?;
    Ok(Json(entries))
}
