use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

#[derive(Debug, Deserialize)]
pub struct MatchRow {
    pub id: Thing,
    pub game_id: String,
    pub tournament_id: Thing,
    pub participants: Vec<MatchParticipant>,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MatchParticipant {
    pub submission_id: Thing,
    pub user_id: Thing,
}

#[derive(Debug, Deserialize)]
pub struct Submission {
    pub id: Thing,
    pub compiled_binary_path: Option<String>,
}

pub struct DbClient {
    db: Surreal<Client>,
}

impl DbClient {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn try_claim_match(&self, match_id: Thing) -> Result<bool> {
        let mut response = self
            .db
            .query(
                "UPDATE $match_id
                 SET status = 'queued'
                 WHERE status = 'pending'
                 RETURN AFTER",
            )
            .bind(("match_id", match_id))
            .await?;

        let updated: Vec<MatchRow> = response.take(0)?;
        Ok(!updated.is_empty())
    }

    pub async fn get_submission(&self, submission_id: Thing) -> Result<Submission> {
        let key = (submission_id.tb.as_str(), submission_id.id.to_string());
        let submission: Option<Submission> = self.db.select(key).await?;
        submission.ok_or_else(|| anyhow::anyhow!("Submission not found"))
    }

    pub async fn update_match_running(&self, match_id: Thing) -> Result<()> {
        self.db
            .query(
                "UPDATE $match_id
                 SET status = 'running',
                     started_at = time::now()",
            )
            .bind(("match_id", match_id))
            .await?;
        Ok(())
    }

    pub async fn update_match_completed(
        &self,
        match_id: Thing,
        results: Vec<ParticipantResult>,
    ) -> Result<()> {
        // Update match status
        self.db
            .query(
                "UPDATE $match_id
                 SET status = 'completed',
                     completed_at = time::now(),
                     participants = $results",
            )
            .bind(("match_id", match_id.clone()))
            .bind(("results", &results))
            .await?;

        // Update tournament participant scores
        for result in results {
            self.db
                .query(
                    "UPDATE tournament_participant
                     SET score = score + $delta
                     WHERE tournament_id = (SELECT tournament_id FROM $match_id)[0]
                       AND submission_id = $submission_id",
                )
                .bind(("match_id", &match_id))
                .bind(("submission_id", result.submission_id))
                .bind(("delta", result.score))
                .await?;
        }

        Ok(())
    }

    pub async fn update_match_failed(&self, match_id: Thing, error: String) -> Result<()> {
        self.db
            .query(
                "UPDATE $match_id
                 SET status = 'failed',
                     completed_at = time::now(),
                     error_message = $error",
            )
            .bind(("match_id", match_id))
            .bind(("error", error))
            .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ParticipantResult {
    pub submission_id: Thing,
    pub user_id: Thing,
    pub score: i32,
    pub error_code: Option<String>,
}
