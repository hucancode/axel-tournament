use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        game::find_game_by_id,
        matches::{Match, MatchParticipant, MatchStatus},
        submission::Submission,
    },
};
use serde::Deserialize;
use std::collections::HashSet;
use surrealdb::sql::{Datetime, Thing};

pub async fn create_match(
    db: &Database,
    tournament_id: Thing,
    game_id: String,
    participant_submission_ids: Vec<Thing>,
) -> ApiResult<Match> {
    // 1. Verify Game exists in hardcoded registry
    find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    // 2. Fetch and Validate Submissions
    let mut participants = Vec::new();

    for sub_id in participant_submission_ids {
        let submission_key = (sub_id.tb.as_str(), sub_id.id.to_string());
        let submission: Option<Submission> = db.select(submission_key).await?;
        let submission = submission
            .ok_or_else(|| ApiError::NotFound(format!("Submission {} not found", sub_id)))?;

        // Ensure submission belongs to the correct game
        if submission.game_id != game_id {
            return Err(ApiError::BadRequest(format!(
                "Submission {} does not belong to game {}",
                sub_id, game_id
            )));
        }
        participants.push(MatchParticipant {
            user_id: None,
            submission_id: Some(submission.id.unwrap()),
            score: None,
            metadata: None,
        });
    }

    // 3. Create Match
    let new_match = Match {
        id: None,
        tournament_id: Some(tournament_id),
        game_id: game_id.clone(),
        status: MatchStatus::Pending,
        participants,
        metadata: None,
        room_id: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: None,
    };

    let created: Option<Match> = db.create("match").content(new_match).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create match".to_string()))
}

pub async fn get_match(db: &Database, match_id: Thing) -> ApiResult<Match> {
    let key = (match_id.tb.as_str(), match_id.id.to_string());
    let match_data: Option<Match> = db.select(key).await?;
    match_data.ok_or_else(|| ApiError::NotFound("Match not found".to_string()))
}

pub async fn list_matches(
    db: &Database,
    tournament_id: Option<Thing>,
    game_id: Option<Thing>,
    user_id: Option<Thing>, // Filter by user involved
    limit: Option<u32>,
    offset: Option<u32>,
) -> ApiResult<Vec<Match>> {
    let mut query = "SELECT * FROM match WHERE 1=1".to_string();
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);

    let tournament_filter = tournament_id.clone();
    let game_filter = game_id.clone();

    if tournament_id.is_some() {
        query.push_str(" AND tournament_id = $tournament_id");
    }
    if game_id.is_some() {
        query.push_str(" AND game_id = $game_id");
    }

    query.push_str(" ORDER BY created_at DESC");
    if user_id.is_none() {
        query.push_str(" LIMIT $limit START $offset");
    }
    let mut db_query = db.query(query);
    if let Some(tid) = tournament_id {
        db_query = db_query.bind(("tournament_id", tid));
    }
    if let Some(gid) = game_id {
        db_query = db_query.bind(("game_id", gid));
    }
    if user_id.is_none() {
        db_query = db_query.bind(("limit", limit)).bind(("offset", offset));
    }
    let mut result = db_query.await?;
    let mut matches: Vec<Match> = result.take(0)?;

    // Optional filtering by user_id through their submissions
    if let Some(uid) = user_id {
        let mut submissions_sql =
            "SELECT id FROM submission WHERE user_id = $user_id".to_string();

        if tournament_filter.is_some() {
            submissions_sql.push_str(" AND tournament_id = $tournament_id");
        }
        if game_filter.is_some() {
            submissions_sql.push_str(" AND game_id = $game_id");
        }

        let mut submissions_query = db.query(submissions_sql).bind(("user_id", uid));
        if let Some(tid) = tournament_filter {
            submissions_query = submissions_query.bind(("tournament_id", tid));
        }
        if let Some(gid) = game_filter {
            submissions_query = submissions_query.bind(("game_id", gid));
        }

        #[derive(Deserialize)]
        struct SubmissionRow {
            id: Thing,
        }

        let mut submission_rows = submissions_query.await?;
        let submissions: Vec<SubmissionRow> = submission_rows.take(0)?;
        let submission_ids: HashSet<Thing> =
            submissions.into_iter().map(|row| row.id).collect();

        if submission_ids.is_empty() {
            return Ok(Vec::new());
        }

        matches.retain(|m| {
            m.participants
                .iter()
                .any(|p| p.submission_id.as_ref().map_or(false, |id| submission_ids.contains(id)))
        });

        if offset as usize >= matches.len() {
            return Ok(Vec::new());
        }
        matches = matches
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
    }

    Ok(matches)
}
