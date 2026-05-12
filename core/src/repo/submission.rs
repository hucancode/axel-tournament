use async_trait::async_trait;
use surrealdb::types::RecordId;
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::submission::{Submission, SubmissionStatus};

#[async_trait]
pub trait SubmissionRepo: Send + Sync {
    async fn get_by_id(&self, submission_id: &RecordId) -> AppResult<Submission>;
    async fn list_for_user(
        &self,
        user_id: &RecordId,
        tournament_id: Option<&RecordId>,
    ) -> AppResult<Vec<Submission>>;
    async fn update_status(
        &self,
        submission_id: &RecordId,
        status: SubmissionStatus,
        error_message: Option<String>,
    ) -> AppResult<Submission>;
}

#[async_trait]
impl<C: Connection> SubmissionRepo for Surreal<C> {
    async fn get_by_id(&self, submission_id: &RecordId) -> AppResult<Submission> {
        let s: Option<Submission> = self.select(submission_id).await?;
        s.ok_or_else(|| AppError::NotFound("Submission not found".into()))
    }

    async fn list_for_user(
        &self,
        user_id: &RecordId,
        tournament_id: Option<&RecordId>,
    ) -> AppResult<Vec<Submission>> {
        let mut result = if let Some(tid) = tournament_id {
            self.query(
                "SELECT * FROM submission
                 WHERE user_id = $user_id AND tournament_id = $tournament_id
                 ORDER BY created_at DESC",
            )
            .bind(("user_id", user_id.clone()))
            .bind(("tournament_id", tid.clone()))
            .await?
        } else {
            self.query("SELECT * FROM submission WHERE user_id = $user_id ORDER BY created_at DESC")
                .bind(("user_id", user_id.clone()))
                .await?
        };
        Ok(result.take(0)?)
    }

    async fn update_status(
        &self,
        submission_id: &RecordId,
        status: SubmissionStatus,
        error_message: Option<String>,
    ) -> AppResult<Submission> {
        let status_tag = serde_json::to_string(&status)
            .expect("enum tag serialisation infallible")
            .trim_matches('"')
            .to_string();
        let mut result = self
            .query("UPDATE $submission_id SET status = $status, error_message = $error")
            .bind(("submission_id", submission_id.clone()))
            .bind(("status", status_tag))
            .bind(("error", error_message))
            .await?;
        let submissions: Vec<Submission> = result.take(0)?;
        submissions
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Submission not found".into()))
    }
}
