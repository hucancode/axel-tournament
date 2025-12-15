use crate::{db::Database, error::ApiResult, models::LeaderboardEntry};
use surrealdb::sql::Thing;

pub async fn get_leaderboard(
    db: &Database,
    limit: u32,
    tournament_id: Option<&str>,
    game_id: Option<&str>,
) -> ApiResult<Vec<LeaderboardEntry>> {
    let limit = limit.min(1000); // Cap at 1000
    let query = if let Some(_tid) = tournament_id {
        "SELECT id, score, user_id, tournament_id,
                user_id.username AS username,
                user_id.location AS location,
                tournament_id.name AS tournament_name
         FROM tournament_participant
         WHERE tournament_id = $tournament_id
         ORDER BY score DESC
         LIMIT $limit"
    } else if let Some(_gid) = game_id {
        "SELECT id, score, user_id, tournament_id,
                user_id.username AS username,
                user_id.location AS location,
                tournament_id.name AS tournament_name
         FROM tournament_participant
         WHERE tournament_id.game_id = $game_id
         ORDER BY score DESC
         LIMIT $limit"
    } else {
        "SELECT id, score, user_id, tournament_id,
                user_id.username AS username,
                user_id.location AS location,
                tournament_id.name AS tournament_name
         FROM tournament_participant
         ORDER BY score DESC
         LIMIT $limit"
    };
    let mut result = db.query(query).bind(("limit", limit));
    if let Some(tid) = tournament_id {
        result = result.bind(("tournament_id", Thing::from(("tournament", tid))));
    }
    if let Some(gid) = game_id {
        result = result.bind(("game_id", Thing::from(("game", gid))));
    }
    let mut response = result.await?;
    #[derive(serde::Deserialize)]
    struct RawEntry {
        id: Option<Thing>,
        user_id: Thing,
        tournament_id: Thing,
        score: f64,
        username: Option<String>,
        location: Option<String>,
        tournament_name: Option<String>,
    }
    let raw_entries: Vec<RawEntry> = response.take(0)?;
    let entries = raw_entries
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| LeaderboardEntry {
            rank: (idx + 1) as u32,
            user_id: entry.user_id.to_string(),
            username: entry.username.unwrap_or_default(),
            location: entry.location.unwrap_or_default(),
            score: entry.score,
            tournament_name: entry.tournament_name.unwrap_or_default(),
            tournament_id: entry.tournament_id.to_string(),
        })
        .collect();
    Ok(entries)
}
