use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        Game,
        matches::{Match, MatchParticipant, MatchParticipantResult, MatchStatus},
        submission::Submission,
    },
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_match(
    db: &Database,
    tournament_id: Option<String>,
    game_id: String,
    participant_submission_ids: Vec<String>,
) -> ApiResult<Match> {
    let game_id_clean = game_id.trim_start_matches("game:").to_string();
    // 1. Verify Game exists
    let game: Option<Game> = db.select(("game", &game_id_clean)).await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    // 2. Fetch and Validate Submissions
    let mut participants = Vec::new();

    for sub_id in participant_submission_ids {
        let submission: Option<Submission> = db.select(("submission", &sub_id)).await?;
        let submission = submission
            .ok_or_else(|| ApiError::NotFound(format!("Submission {} not found", sub_id)))?;

        // Ensure submission belongs to the correct game
        // Note: Thing comparison handles equality correctly
        if submission.game_id != game.id.clone().unwrap() {
            return Err(ApiError::BadRequest(format!(
                "Submission {} does not belong to game {}",
                sub_id, game_id_clean
            )));
        }
        participants.push(MatchParticipant {
            submission_id: submission.id.unwrap(),
            user_id: submission.user_id,
            score: None,
            rank: None,
            is_winner: false,
            metadata: None,
        });
    }

    // 3. Create Match
    let new_match = Match {
        id: None,
        tournament_id: tournament_id
            .map(|id| Thing::from(("tournament", id.trim_start_matches("tournament:")))),
        game_id: game.id.unwrap(),
        status: MatchStatus::Pending,
        participants,
        metadata: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: None,
    };

    let created: Option<Match> = db.create("match").content(new_match).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create match".to_string()))
}

pub async fn get_match(db: &Database, match_id: &str) -> ApiResult<Match> {
    let match_data: Option<Match> = db.select(("match", match_id)).await?;
    match_data.ok_or_else(|| ApiError::NotFound("Match not found".to_string()))
}

pub async fn list_matches(
    db: &Database,
    tournament_id: Option<String>,
    game_id: Option<String>,
    user_id: Option<String>, // Filter by user involved
) -> ApiResult<Vec<Match>> {
    let mut query = "SELECT * FROM match WHERE 1=1".to_string();

    if let Some(tid) = tournament_id {
        query.push_str(&format!(
            " AND tournament_id = type::thing('tournament', '{}')",
            tid
        ));
    }
    if let Some(gid) = game_id {
        query.push_str(&format!(" AND game_id = type::thing('game', '{}')", gid));
    }
    // Filtering by user_id in participants array
    if let Some(uid) = user_id {
        query.push_str(&format!(
            " AND participants[?].user_id CONTAINS type::thing('user', '{}')",
            uid
        ));
    }

    query.push_str(" ORDER BY created_at DESC");
    let mut result = db.query(query).await?;
    let matches: Vec<Match> = result.take(0)?;
    Ok(matches)
}

pub async fn update_match_result(
    db: &Database,
    match_id: &str,
    status: MatchStatus,
    participants_results: Vec<MatchParticipantResult>,
    metadata: Option<serde_json::Value>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
) -> ApiResult<Match> {
    let mut match_data = get_match(db, match_id).await?;

    match_data.status = status;
    if let Some(m) = metadata {
        match_data.metadata = Some(m);
    }

    if let Some(sa) = started_at {
        match_data.started_at = Some(sa.into());
    }
    if let Some(ca) = completed_at {
        match_data.completed_at = Some(ca.into());
    }
    // Update participants
    for res in participants_results {
        // Construct Thing from submission_id string (assuming it's just the ID part)
        let target_thing = Thing::from(("submission", res.submission_id.as_str()));

        if let Some(p) = match_data
            .participants
            .iter_mut()
            .find(|p| p.submission_id == target_thing)
        {
            p.score = Some(res.score);
            p.rank = res.rank;
            p.is_winner = res.is_winner;
            p.metadata = res.metadata;
        }
    }

    match_data.updated_at = Datetime::default();

    let updated: Option<Match> = db.update(("match", match_id)).content(match_data).await?;
    updated.ok_or_else(|| ApiError::Internal("Failed to update match".to_string()))
}
