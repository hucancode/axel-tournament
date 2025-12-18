use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::{Datetime, Thing};

#[derive(Clone)]
pub struct DbClient {
    db: Arc<Surreal<Client>>,
}

impl DbClient {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }

    pub async fn fetch_game(&self, game_id: &str) -> Result<Game> {
        let game_thing: Thing = game_id
            .parse()
            .map_err(|_| anyhow!("Invalid game id {}", game_id))?;
        let key = (game_thing.tb.as_str(), game_thing.id.to_string());
        let game: Option<Game> = self.db.select(key).await?;
        game.context(format!("Game {} not found", game_id))
    }

    pub async fn update_match_status(&self, match_id: &str, status: &str) -> Result<()> {
        let match_thing: Thing = match_id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_id))?;
        let query = if status == "running" {
            "UPDATE $match_id SET status = $status, started_at = time::now(), updated_at = time::now() RETURN *"
        } else {
            "UPDATE $match_id SET status = $status, updated_at = time::now() RETURN *"
        };

        let mut response = self
            .db
            .query(query)
            .bind(("match_id", match_thing))
            .bind(("status", status.to_string()))
            .await?;

        let updated: Option<Value> = response.take(0)?;
        updated.context("Failed to update match status")?;
        Ok(())
    }

    pub async fn report_match_result(&self, match_data: &Match, result: MatchResult) -> Result<()> {
        let match_id: Thing = match_data
            .id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_data.id))?;
        let started_at = Datetime::from(result.started_at);
        let completed_at = Datetime::from(result.completed_at);

        let mut participants = match_data.participants.clone();
        for participant in participants.iter_mut() {
            if let Some(res) = result
                .participants
                .iter()
                .find(|r| r.submission_id == participant.submission_id)
            {
                participant.score = Some(res.score);
                participant.rank = res.rank;
                participant.is_winner = res.is_winner;
                participant.metadata = res.metadata.clone();
            }
        }

        let mut response = self
            .db
            .query(
                "UPDATE $match_id SET
                    status = 'completed',
                    participants = $participants,
                    metadata = $metadata,
                    started_at = $started_at,
                    completed_at = $completed_at,
                    updated_at = time::now()
                 RETURN *",
            )
            .bind(("match_id", match_id.clone()))
            .bind(("participants", participants))
            .bind(("metadata", result.metadata))
            .bind(("started_at", started_at))
            .bind(("completed_at", completed_at))
            .await?;

        let updated: Option<Value> = response.take(0)?;
        updated.context("Failed to persist match result")?;

        self.update_tournament_scores(match_data, &result.participants)
            .await?;

        Ok(())
    }

    pub async fn report_match_failure(&self, match_id: &str, error_msg: &str) -> Result<()> {
        let match_id: Thing = match_id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_id))?;

        let mut response = self
            .db
            .query(
                "UPDATE $match_id SET
                    status = 'failed',
                    metadata = $metadata,
                    completed_at = time::now(),
                    updated_at = time::now()
                 RETURN *",
            )
            .bind(("match_id", match_id))
            .bind(("metadata", json!({ "error": error_msg })))
            .await?;

        let updated: Option<Value> = response.take(0)?;
        updated.context("Failed to mark match as failed")?;

        Ok(())
    }

    async fn update_tournament_scores(
        &self,
        match_data: &Match,
        results: &[ParticipantResult],
    ) -> Result<()> {
        let Some(tournament_id) = match_data.tournament_id.as_ref() else {
            return Ok(());
        };

        let tournament_thing: Thing = tournament_id
            .parse()
            .map_err(|_| anyhow!("Invalid tournament id {}", tournament_id))?;

        for result in results {
            let submission_thing: Thing = result
                .submission_id
                .parse()
                .map_err(|_| anyhow!("Invalid submission id {}", result.submission_id))?;

            #[derive(Deserialize)]
            struct ScoreRow {
                score: Option<f64>,
            }

            let mut current_score = self
                .db
                .query(
                    "SELECT score FROM tournament_participant
                     WHERE tournament_id = $tournament_id AND submission_id = $submission_id",
                )
                .bind(("tournament_id", tournament_thing.clone()))
                .bind(("submission_id", submission_thing.clone()))
                .await?;

            let existing: Vec<ScoreRow> = current_score.take(0)?;
            let base = existing.first().and_then(|row| row.score).unwrap_or(0.0);
            let new_score = base + result.score;

            self.db
                .query(
                    "UPDATE tournament_participant
                     SET score = $score
                     WHERE tournament_id = $tournament_id AND submission_id = $submission_id",
                )
                .bind(("tournament_id", tournament_thing.clone()))
                .bind(("submission_id", submission_thing))
                .bind(("score", new_score))
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Match {
    pub id: String,
    pub game_id: String,
    #[serde(default)]
    pub tournament_id: Option<String>,
    pub status: String,
    pub participants: Vec<MatchParticipant>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MatchParticipant {
    pub submission_id: String,
    pub user_id: String,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub rank: Option<u32>,
    #[serde(default)]
    pub is_winner: bool,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Game {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub game_code: Option<String>,
    #[serde(default)]
    pub game_language: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchResult {
    pub participants: Vec<ParticipantResult>,
    pub metadata: Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParticipantResult {
    pub submission_id: String,
    pub score: f64,
    pub rank: Option<u32>,
    pub is_winner: bool,
    pub metadata: Option<Value>,
}
