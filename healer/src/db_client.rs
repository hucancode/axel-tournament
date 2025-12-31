use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

#[derive(Debug, Deserialize)]
pub struct SubmissionRow {
    pub id: Thing,
    pub game_id: String,
    pub language: String,
    pub code: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct CompilationResult {
    pub status: String,
    pub compiled_binary_path: Option<String>,
    pub error_message: Option<String>,
}

pub struct DbClient {
    db: Surreal<Client>,
}

impl DbClient {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn try_claim_submission(&self, submission_id: Thing) -> Result<bool> {
        let mut response = self
            .db
            .query(
                "UPDATE $submission_id
                 SET status = 'compiling'
                 WHERE status = 'pending'
                 RETURN AFTER",
            )
            .bind(("submission_id", submission_id))
            .await?;

        let updated: Vec<SubmissionRow> = response.take(0)?;
        Ok(!updated.is_empty())
    }

    pub async fn update_compilation_success(
        &self,
        submission_id: Thing,
        binary_path: String,
    ) -> Result<()> {
        self.db
            .query(
                "UPDATE $submission_id
                 SET status = 'accepted',
                     compiled_binary_path = $binary_path,
                     error_message = NONE",
            )
            .bind(("submission_id", submission_id))
            .bind(("binary_path", binary_path))
            .await?;

        Ok(())
    }

    pub async fn update_compilation_failure(
        &self,
        submission_id: Thing,
        error_message: String,
    ) -> Result<()> {
        self.db
            .query(
                "UPDATE $submission_id
                 SET status = 'failed',
                     error_message = $error_message,
                     compiled_binary_path = NONE",
            )
            .bind(("submission_id", submission_id))
            .bind(("error_message", error_message))
            .await?;

        Ok(())
    }
}
