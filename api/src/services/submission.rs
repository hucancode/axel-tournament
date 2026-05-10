use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{ProgrammingLanguage, Submission, SubmissionStatus, TournamentParticipant},
    services::common::enum_tag,
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

    // Participant check first — non-participants can't trip the rate limit.
    let participant = crate::services::tournament::get_participant(db, &tournament_id, &user_id)
        .await?
        .ok_or_else(|| {
            ApiError::Forbidden("You must join the tournament before submitting code".to_string())
        })?;

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

    let now = Datetime::default();
    let mut result = db
        .query(
            "LET $submission = (CREATE submission SET
                 user_id = $user_id, tournament_id = $tournament_id,
                 game_id = $game_id, language = $language, code = $code,
                 status = $status, error_message = NONE, created_at = $now
                 RETURN AFTER);
             IF $participant_submission_id = NONE THEN
                UPDATE $participant_id SET submission_id = $submission[0].id;
             END;
             RETURN $submission;",
        )
        .bind(("user_id", user_id))
        .bind(("tournament_id", tournament_id))
        .bind(("game_id", game_id))
        .bind(("language", enum_tag(&language)))
        .bind(("code", code))
        .bind(("status", enum_tag(&SubmissionStatus::Pending)))
        .bind(("now", now))
        .bind(("participant_id", participant.id.clone()))
        .bind(("participant_submission_id", participant.submission_id.clone()))
        .await?
        .check()?;
    let submissions: Vec<Submission> = result.take(2)?;
    submissions
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::Internal("Submission create returned no row".to_string()))
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
            .bind(("user_id", user_id))
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
    let rows: Vec<TournamentParticipant> = updated.take(0)?;
    if rows.is_empty() {
        return Err(ApiError::Forbidden(
            "You must join the tournament before selecting a bot".to_string(),
        ));
    }
    Ok(submission)
}

pub async fn update_submission_status(
    db: &Database,
    submission_id: RecordId,
    status: SubmissionStatus,
    error_message: Option<String>,
) -> ApiResult<Submission> {
    let mut result = db
        .query("UPDATE $submission_id SET status = $status, error_message = $error")
        .bind(("submission_id", submission_id))
        .bind(("status", enum_tag(&status)))
        .bind(("error", error_message))
        .await?;
    let submissions: Vec<Submission> = result.take(0)?;
    submissions
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))
}
