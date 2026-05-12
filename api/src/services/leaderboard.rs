use crate::{
    db::Database,
    error::AppResult,
    models::{LeaderboardEntry, bare_key},
};
use surrealdb::types::{RecordId, SurrealValue};

const MAX_LIMIT: u32 = 1000;

const SELECT_CLAUSE: &str = "SELECT id, score, user_id, tournament_id,
       user_id.username AS username,
       user_id.location AS location,
       tournament_id.name AS tournament_name
FROM tournament_participant WHERE tournament_id = $tournament_id
ORDER BY score DESC LIMIT $limit";

pub async fn get_leaderboard(
    db: &Database,
    tournament_id: RecordId,
    limit: u32,
) -> AppResult<Vec<LeaderboardEntry>> {
    let limit = limit.min(MAX_LIMIT);
    let mut response = db
        .query(SELECT_CLAUSE)
        .bind(("tournament_id", tournament_id))
        .bind(("limit", limit))
        .await?;

    #[derive(serde::Deserialize, SurrealValue)]
    struct RawEntry {
        user_id: RecordId,
        tournament_id: RecordId,
        score: f64,
        username: Option<String>,
        location: Option<String>,
        tournament_name: Option<String>,
    }
    let raw: Vec<RawEntry> = response.take(0)?;
    Ok(raw
        .into_iter()
        .enumerate()
        .map(|(idx, e)| LeaderboardEntry {
            rank: (idx + 1) as u32,
            user_id: bare_key(&e.user_id),
            username: e.username.unwrap_or_default(),
            location: e.location.unwrap_or_default(),
            score: e.score,
            tournament_name: e.tournament_name.unwrap_or_default(),
            tournament_id: bare_key(&e.tournament_id),
        })
        .collect())
}
