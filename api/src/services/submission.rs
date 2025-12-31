use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{ProgrammingLanguage, Submission, SubmissionStatus},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_submission(
    db: &Database,
    user_id: Thing,
    tournament_id: Thing,
    game_id: String,
    language: ProgrammingLanguage,
    code: String,
) -> ApiResult<Submission> {
    let status = serde_json::to_string(&SubmissionStatus::Pending)
        .unwrap()
        .trim_matches('"')
        .to_string();
    let now = Datetime::default();
    let mut result = db
        .query(
            "LET $participant = (SELECT id FROM tournament_participant WHERE tournament_id = $tournament_id AND user_id = $user_id LIMIT 1);
             LET $submission = (IF array::len($participant) = 0 THEN [] ELSE
                (CREATE submission SET user_id = $user_id, tournament_id = $tournament_id,
                 game_id = $game_id, language = $language, code = $code, status = $status,
                 error_message = NONE, created_at = $now RETURN AFTER) END);
             IF array::len($participant) > 0 THEN
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

pub async fn get_submission(db: &Database, submission_id: Thing) -> ApiResult<Submission> {
    let key = (submission_id.tb.as_str(), submission_id.id.to_string());
    let submission: Option<Submission> = db.select(key).await?;
    submission.ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))
}

pub async fn list_user_submissions(
    db: &Database,
    user_id: Thing,
    tournament_id: Option<Thing>,
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

pub async fn update_submission_status(
    db: &Database,
    submission_id: Thing,
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
