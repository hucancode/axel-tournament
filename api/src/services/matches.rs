use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        game::find_game_by_id,
        matches::{Match, MatchParticipant},
        submission::Submission,
    },
};
use surrealdb::types::{RecordId, ToSql};

pub async fn create_match(
    db: &Database,
    tournament_id: RecordId,
    game_id: String,
    participant_submission_ids: Vec<RecordId>,
) -> ApiResult<Match> {
    find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    let mut participants = Vec::new();
    for sub_id in participant_submission_ids {
        let submission: Option<Submission> = db.select(&sub_id).await?;
        let submission = submission.ok_or_else(|| {
            ApiError::NotFound(format!("Submission {} not found", sub_id.to_sql()))
        })?;
        if submission.game_id != game_id {
            return Err(ApiError::BadRequest(format!(
                "Submission {} does not belong to game {}",
                sub_id.to_sql(),
                game_id
            )));
        }
        participants.push(MatchParticipant {
            user_id: submission.user_id.clone(),
            submission_id: submission.id.clone(),
            score: None,
        });
    }

    let new_match = Match {
        tournament_id: Some(tournament_id),
        game_id: game_id.clone(),
        participants,
        ..Default::default()
    };

    let created: Option<Match> = db.create("match").content(new_match).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create match".to_string()))
}

pub async fn get_match(db: &Database, match_id: RecordId) -> ApiResult<Match> {
    let match_data: Option<Match> = db.select(&match_id).await?;
    match_data.ok_or_else(|| ApiError::NotFound("Match not found".to_string()))
}

/// List matches with optional filters. Pagination happens DB-side.
/// `user_id` filter joins via `submission.user_id` on the SurrealQL side
/// so `LIMIT` / `START` apply to the joined rows, not a post-fetch filter.
pub async fn list_matches(
    db: &Database,
    tournament_id: Option<RecordId>,
    game_id: Option<RecordId>,
    user_id: Option<RecordId>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> ApiResult<Vec<Match>> {
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);

    let mut where_parts: Vec<&str> = Vec::new();
    if tournament_id.is_some() {
        where_parts.push("tournament_id = $tournament_id");
    }
    if game_id.is_some() {
        where_parts.push("game_id = $game_id");
    }
    if user_id.is_some() {
        where_parts
            .push("participants[*].submission_id ANYINSIDE (SELECT VALUE id FROM submission WHERE user_id = $user_id)");
    }
    let where_clause = if where_parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_parts.join(" AND "))
    };
    let query = format!(
        "SELECT * FROM match{} ORDER BY created_at DESC LIMIT $limit START $offset",
        where_clause
    );

    let mut q = db
        .query(query)
        .bind(("limit", limit))
        .bind(("offset", offset));
    if let Some(tid) = tournament_id {
        q = q.bind(("tournament_id", tid));
    }
    if let Some(gid) = game_id {
        q = q.bind(("game_id", gid));
    }
    if let Some(uid) = user_id {
        q = q.bind(("user_id", uid));
    }
    let mut result = q.await?;
    let matches: Vec<Match> = result.take(0)?;
    Ok(matches)
}
