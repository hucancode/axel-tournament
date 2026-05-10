use crate::{
    db::Database,
    error::ApiResult,
    models::{LeaderboardEntry, bare_key},
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

pub async fn get_leaderboard(
    db: &Database,
    limit: u32,
    tournament_id: Option<RecordId>,
    game_id: Option<RecordId>,
) -> ApiResult<Vec<LeaderboardEntry>> {
    let limit = limit.min(1000); // Cap at 1000
    let tournament_id_val = tournament_id.as_ref().map(|t| t.to_sql());
    let game_id_val = game_id.as_ref().map(|g| g.to_sql());
    let query = if let Some(_tid) = tournament_id_val {
        "SELECT id, score, user_id, tournament_id,
                user_id.username AS username,
                user_id.location AS location,
                tournament_id.name AS tournament_name
         FROM tournament_participant
         WHERE tournament_id = $tournament_id
         ORDER BY score DESC
         LIMIT $limit"
    } else if let Some(_gid) = game_id_val {
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
    if let Some(tid) = tournament_id.clone() {
        result = result.bind(("tournament_id", tid));
    }
    if let Some(gid) = game_id.clone() {
        result = result.bind(("game_id", gid));
    }
    let mut response = result.await?;
    #[derive(serde::Deserialize, SurrealValue)]
    struct RawEntry {
        user_id: RecordId,
        tournament_id: RecordId,
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
            user_id: bare_key(&entry.user_id),
            username: entry.username.unwrap_or_default(),
            location: entry.location.unwrap_or_default(),
            score: entry.score,
            tournament_name: entry.tournament_name.unwrap_or_default(),
            tournament_id: bare_key(&entry.tournament_id),
        })
        .collect();
    Ok(entries)
}
