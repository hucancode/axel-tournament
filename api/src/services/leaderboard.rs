use crate::{
    db::Database,
    error::ApiResult,
    models::{LeaderboardEntry, bare_key},
};
use surrealdb::types::{RecordId, SurrealValue};

const MAX_LIMIT: u32 = 1000;

const SELECT_CLAUSE: &str = "SELECT id, score, user_id, tournament_id,
       user_id.username AS username,
       user_id.location AS location,
       tournament_id.name AS tournament_name
FROM tournament_participant";

pub async fn get_leaderboard(
    db: &Database,
    limit: u32,
    tournament_id: Option<RecordId>,
    game_id: Option<RecordId>,
) -> ApiResult<Vec<LeaderboardEntry>> {
    let limit = limit.min(MAX_LIMIT);
    let where_clause = if tournament_id.is_some() {
        " WHERE tournament_id = $tournament_id"
    } else if game_id.is_some() {
        " WHERE tournament_id.game_id = $game_id"
    } else {
        ""
    };
    let query = format!(
        "{}{} ORDER BY score DESC LIMIT $limit",
        SELECT_CLAUSE, where_clause
    );

    let mut q = db.query(query).bind(("limit", limit));
    if let Some(tid) = tournament_id {
        q = q.bind(("tournament_id", tid));
    }
    if let Some(gid) = game_id {
        q = q.bind(("game_id", gid));
    }
    let mut response = q.await?;

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
