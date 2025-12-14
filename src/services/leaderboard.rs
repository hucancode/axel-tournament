use crate::{
    db::Database,
    error::ApiResult,
    models::LeaderboardEntry,
};
use surrealdb::sql::Thing;

pub async fn get_leaderboard(
    db: &Database,
    limit: u32,
    tournament_id: Option<&str>,
    game_id: Option<&str>,
) -> ApiResult<Vec<LeaderboardEntry>> {
    let limit = limit.min(1000); // Cap at 1000

    let query = if let Some(_tid) = tournament_id {
        "SELECT *, user_id.*, tournament_id.*
        FROM tournament_participant
        WHERE tournament_id = $tournament_id
        ORDER BY score DESC
        LIMIT $limit
        FETCH user_id, tournament_id"
    } else if let Some(_gid) = game_id {
        "SELECT *, user_id.*, tournament_id.*, tournament_id.game_id.*
        FROM tournament_participant
        WHERE tournament_id.game_id = $game_id
        ORDER BY score DESC
        LIMIT $limit
        FETCH user_id, tournament_id"
    } else {
        "SELECT *, user_id.*, tournament_id.*
        FROM tournament_participant
        ORDER BY score DESC
        LIMIT $limit
        FETCH user_id, tournament_id"
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
    struct UserRecord {
        username: String,
        location: String,
    }

    #[derive(serde::Deserialize)]
    struct TournamentRecord {
        name: String,
    }

    #[derive(serde::Deserialize)]
    struct RawEntry {
        user_id: UserRecord,
        score: f64,
        tournament_id: TournamentRecord,
        id: Option<Thing>,
    }

    let raw_entries: Vec<RawEntry> = response.take(0)?;

    let entries = raw_entries
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| LeaderboardEntry {
            rank: (idx + 1) as u32,
            user_id: entry.id.as_ref().map(|t| t.to_string()).unwrap_or_default(),
            username: entry.user_id.username,
            location: entry.user_id.location,
            score: entry.score,
            tournament_name: entry.tournament_id.name,
            tournament_id: entry.id.map(|t| t.to_string()).unwrap_or_default(),
        })
        .collect();

    Ok(entries)
}
