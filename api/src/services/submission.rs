use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{ProgrammingLanguage, Submission, SubmissionStatus},
};
use surrealdb::types::{Datetime, RecordId};

/// Maximum submissions a user may create within `RATE_WINDOW`. Bots
/// upload often during dev, but more than this looks like spam.
pub const RATE_LIMIT: u32 = 10;
pub const RATE_WINDOW_SECS: u32 = 60;

/// Maximum source-code size (bytes) we accept for a single submission.
/// Real bots fit comfortably; anything larger is either a mistake or an
/// attempt to push the DB / compiler over a cliff.
pub const MAX_CODE_BYTES: usize = 256 * 1024;

pub async fn create_submission(
    db: &Database,
    user_id: RecordId,
    tournament_id: RecordId,
    game_id: String,
    language: ProgrammingLanguage,
    code: String,
) -> ApiResult<Submission> {
    if code.len() > MAX_CODE_BYTES {
        return Err(ApiError::BadRequest(format!(
            "Submission too large: {} bytes (max {} bytes / {} KiB).",
            code.len(),
            MAX_CODE_BYTES,
            MAX_CODE_BYTES / 1024,
        )));
    }
    // Rate-limit by recent submission count for this user + tournament.
    let mut count_resp = db
        .query(
            "SELECT count() AS n FROM submission
             WHERE user_id = $uid AND tournament_id = $tid
             AND created_at > time::now() - duration::from_secs($w)
             GROUP ALL",
        )
        .bind(("uid", user_id.clone()))
        .bind(("tid", tournament_id.clone()))
        .bind(("w", RATE_WINDOW_SECS as i64))
        .await?;
    use surrealdb::types::SurrealValue;
    #[derive(serde::Deserialize, SurrealValue)]
    struct CountRow {
        n: i64,
    }
    let rows: Vec<CountRow> = count_resp.take(0).unwrap_or_default();
    drop(count_resp);
    let count = rows.into_iter().next().map(|r| r.n).unwrap_or(0);
    if count as u32 >= RATE_LIMIT {
        return Err(ApiError::BadRequest(format!(
            "Rate limit: max {} submissions per {} seconds for this tournament. Wait and try again.",
            RATE_LIMIT, RATE_WINDOW_SECS
        )));
    }

    let status = serde_json::to_string(&SubmissionStatus::Pending)
        .unwrap()
        .trim_matches('"')
        .to_string();
    let now = Datetime::default();
    let mut result = db
        .query(
            "LET $participant = (SELECT id, submission_id FROM tournament_participant WHERE tournament_id = $tournament_id AND user_id = $user_id LIMIT 1);
             LET $submission = (IF array::len($participant) = 0 THEN [] ELSE
                (CREATE submission SET user_id = $user_id, tournament_id = $tournament_id,
                 game_id = $game_id, language = $language, code = $code, status = $status,
                 error_message = NONE, created_at = $now RETURN AFTER) END);
             IF array::len($participant) > 0 AND $participant[0].submission_id = NONE THEN
                UPDATE $participant[0].id SET submission_id = $submission[0].id;
             END;
             RETURN $submission;",
        )
        .bind(("user_id", user_id))
        .bind(("tournament_id", tournament_id))
        .bind(("game_id", game_id))
        .bind(("language", serde_json::to_string(&language).unwrap().trim_matches('"').to_string()))
        .bind(("code", code))
        .bind(("status", status))
        .bind(("now", now))
        .await?
        .check()?;
    let submissions: Vec<Submission> = result.take(3)?; // take the 4th result
    let submission = submissions
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::Forbidden(
            "You must join the tournament before submitting code".to_string(),
        ))?;
    Ok(submission)
}

pub async fn get_submission(db: &Database, submission_id: RecordId) -> ApiResult<Submission> {
    let submission: Option<Submission> = db.select(&submission_id).await?;
    submission.ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))
}

pub async fn list_user_submissions(
    db: &Database,
    user_id: RecordId,
    tournament_id: Option<RecordId>,
) -> ApiResult<Vec<Submission>> {
    let mut result = if let Some(tid) = tournament_id {
        db.query("SELECT * FROM submission WHERE user_id = $user_id AND tournament_id = $tournament_id ORDER BY created_at DESC")
            .bind(("user_id", user_id.clone()))
            .bind(("tournament_id", tid))
            .await?
    } else {
        db.query("SELECT * FROM submission WHERE user_id = $user_id ORDER BY created_at DESC")
            .bind(("user_id", user_id))
            .await?
    };
    let submissions: Vec<Submission> = result.take(0)?;
    Ok(submissions)
}

/// Mark this submission as the active bot for the user's tournament
/// participant entry. The user must already be a participant and the
/// submission must belong to the same user + tournament.
pub async fn select_active_submission(
    db: &Database,
    user_id: RecordId,
    submission_id: RecordId,
) -> ApiResult<Submission> {
    let submission: Submission = {
        let opt: Option<Submission> = db.select(&submission_id).await?;
        opt.ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))?
    };
    if submission.user_id != user_id {
        return Err(ApiError::Forbidden(
            "You don't have access to this submission".to_string(),
        ));
    }
    let mut updated = db
        .query(
            "UPDATE tournament_participant
             SET submission_id = $sid
             WHERE tournament_id = $tid AND user_id = $uid
             RETURN AFTER;",
        )
        .bind(("sid", submission_id))
        .bind(("tid", submission.tournament_id.clone()))
        .bind(("uid", user_id))
        .await?;
    let rows: Vec<crate::models::TournamentParticipant> = updated.take(0)?;
    if rows.is_empty() {
        return Err(ApiError::Forbidden(
            "You must join the tournament before selecting a bot".to_string(),
        ));
    }
    Ok(submission)
}

/// Aggregate statistics for one submission's bot across every match it
/// played. `wins`/`losses`/`draws` are decided by score comparison (the
/// match's other participant). `total_score` is the absolute sum so
/// scored games (e.g. PD payoff) still surface a meaningful value.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubmissionStats {
    pub submission_id: String,
    pub matches_played: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total_score: f64,
}

pub async fn submission_stats(
    db: &Database,
    submission_id: RecordId,
) -> ApiResult<SubmissionStats> {
    use crate::models::matches::{Match, MatchStatus};
    use surrealdb::types::ToSql;

    let mut resp = db
        .query(
            "SELECT * FROM match
             WHERE participants[*].submission_id CONTAINS $sid
             AND status IN ['completed', 'failed']",
        )
        .bind(("sid", submission_id.clone()))
        .await?;
    let matches: Vec<Match> = resp.take(0)?;

    let sid_sql = submission_id.to_sql();
    let mut stats = SubmissionStats {
        submission_id: sid_sql.clone(),
        matches_played: 0,
        wins: 0,
        losses: 0,
        draws: 0,
        total_score: 0.0,
    };

    for m in matches {
        let me_idx = m.participants.iter().position(|p| {
            p.submission_id
                .as_ref()
                .map(|s| s.to_sql() == sid_sql)
                .unwrap_or(false)
        });
        let Some(me_idx) = me_idx else { continue };
        stats.matches_played += 1;

        if m.status == MatchStatus::Failed {
            stats.losses += 1;
            continue;
        }
        let my_score = m.participants[me_idx].score.unwrap_or(0.0);
        stats.total_score += my_score;

        let other_max = m
            .participants
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != me_idx)
            .filter_map(|(_, p)| p.score)
            .fold(f64::NEG_INFINITY, f64::max);
        if !other_max.is_finite() {
            continue;
        }
        if my_score > other_max {
            stats.wins += 1;
        } else if my_score < other_max {
            stats.losses += 1;
        } else {
            stats.draws += 1;
        }
    }
    Ok(stats)
}

pub async fn update_submission_status(
    db: &Database,
    submission_id: RecordId,
    status: SubmissionStatus,
    error_message: Option<String>,
) -> ApiResult<Submission> {
    let status_str = serde_json::to_string(&status)
        .unwrap()
        .trim_matches('"')
        .to_string();
    let mut result = db
        .query("UPDATE $submission_id SET status = $status, error_message = $error")
        .bind(("submission_id", submission_id))
        .bind(("status", status_str))
        .bind(("error", error_message))
        .await?;
    let submissions: Vec<Submission> = result.take(0)?;
    submissions
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))
}
