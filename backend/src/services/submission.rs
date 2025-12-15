use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{ProgrammingLanguage, Submission, SubmissionStatus},
};
use std::path::Path;
use surrealdb::sql::Datetime;
use surrealdb::sql::Thing;
use tokio::fs;

pub async fn create_submission(
    db: &Database,
    user_id: &str,
    tournament_id: &str,
    game_id: &str,
    language: ProgrammingLanguage,
    code: String,
) -> ApiResult<Submission> {
    let user_id_clean = user_id.trim_start_matches("user:");
    let tournament_id_clean = tournament_id.trim_start_matches("tournament:");
    let game_id_clean = game_id.trim_start_matches("game:");
    // Create uploads directory if it doesn't exist
    let upload_dir = Path::new("uploads");
    if !upload_dir.exists() {
        fs::create_dir_all(upload_dir).await?;
    }
    // Generate file path
    let timestamp = Datetime::default().timestamp();
    let file_name = format!(
        "{}_{}_{}_{}.{}",
        user_id,
        tournament_id,
        timestamp,
        uuid::Uuid::new_v4(),
        language.to_extension()
    );
    let file_path = upload_dir.join(&file_name);
    // Write code to file
    fs::write(&file_path, &code).await?;
    let submission = Submission {
        id: None,
        user_id: Thing::from(("user", user_id_clean)),
        tournament_id: Thing::from(("tournament", tournament_id_clean)),
        game_id: Thing::from(("game", game_id_clean)),
        language,
        code,
        file_path: file_path.to_string_lossy().to_string(),
        status: SubmissionStatus::Pending,
        error_message: None,
        created_at: Datetime::default(),
    };
    let created: Option<Submission> = db.create("submission").content(submission).await?;
    let submission =
        created.ok_or_else(|| ApiError::Internal("Failed to create submission".to_string()))?;
    // Update tournament participant with latest submission
    db.query("UPDATE tournament_participant SET submission_id = $submission_id WHERE tournament_id = $tournament_id AND user_id = $user_id")
        .bind(("submission_id", submission.id.clone().unwrap()))
        .bind(("tournament_id", Thing::from(("tournament", tournament_id))))
        .bind(("user_id", Thing::from(("user", user_id))))
        .await?;
    Ok(submission)
}

pub async fn get_submission(db: &Database, submission_id: &str) -> ApiResult<Submission> {
    let submission: Option<Submission> = db.select(("submission", submission_id)).await?;
    submission.ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))
}

pub async fn list_user_submissions(
    db: &Database,
    user_id: &str,
    tournament_id: Option<&str>,
) -> ApiResult<Vec<Submission>> {
    let mut result = if let Some(tid) = tournament_id {
        let tournament_id_clean = tid.trim_start_matches("tournament:");
        db.query("SELECT * FROM submission WHERE user_id = $user_id AND tournament_id = $tournament_id ORDER BY created_at DESC")
            .bind(("user_id", Thing::from(("user", user_id.trim_start_matches("user:")))))
            .bind(("tournament_id", Thing::from(("tournament", tournament_id_clean))))
            .await?
    } else {
        db.query("SELECT * FROM submission WHERE user_id = $user_id ORDER BY created_at DESC")
            .bind((
                "user_id",
                Thing::from(("user", user_id.trim_start_matches("user:"))),
            ))
            .await?
    };
    let submissions: Vec<Submission> = result.take(0)?;
    Ok(submissions)
}

pub async fn update_submission_status(
    db: &Database,
    submission_id: &str,
    status: SubmissionStatus,
    error_message: Option<String>,
) -> ApiResult<Submission> {
    let status_str = serde_json::to_string(&status)
        .unwrap()
        .trim_matches('"')
        .to_string();
    let mut result = db
        .query("UPDATE $submission_id SET status = $status, error_message = $error")
        .bind(("submission_id", Thing::from(("submission", submission_id))))
        .bind(("status", status_str))
        .bind(("error", error_message))
        .await?;
    let submissions: Vec<Submission> = result.take(0)?;
    submissions
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("Submission not found".to_string()))
}
