use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::{Datetime, Thing};
use surrealdb::Surreal;

#[derive(Clone)]
pub struct DbClient {
    db: Arc<Surreal<Client>>,
}

impl DbClient {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }

    pub async fn fetch_match(&self, match_id: &str) -> Result<Match> {
        let match_thing: Thing = match_id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_id))?;

        // SELECT queries return: id as Thing, but relation fields as strings
        #[derive(Deserialize)]
        struct MatchRecord {
            id: Thing,
            game_id: Thing,
            #[serde(default)]
            tournament_id: Option<Thing>,
            status: String,
            participants: Vec<MatchParticipant>,
        }

        let mut response = self
            .db
            .query("SELECT * FROM $match_id")
            .bind(("match_id", match_thing))
            .await?;

        let records: Vec<MatchRecord> = response.take(0)?;
        let record = records
            .into_iter()
            .next()
            .context(format!("Match {} not found", match_id))?;

        Ok(Match {
            id: record.id.to_string(),
            game_id: record.game_id.to_string(),
            tournament_id: record.tournament_id.map(|t| t.to_string()),
            status: record.status,
            participants: record.participants,
        })
    }

    pub async fn fetch_game(&self, game_id: &str) -> Result<Game> {
        let game_thing: Thing = game_id
            .parse()
            .map_err(|_| anyhow!("Invalid game id {}", game_id))?;

        let mut response = self
            .db
            .query("SELECT * FROM $game_id")
            .bind(("game_id", game_thing))
            .await?;

        let games: Vec<Game> = response.take(0)?;
        games
            .into_iter()
            .next()
            .context(format!("Game {} not found", game_id))
    }

    pub async fn fetch_submission(&self, submission_id: &str) -> Result<Submission> {
        let submission_thing: Thing = submission_id
            .parse()
            .map_err(|_| anyhow!("Invalid submission id {}", submission_id))?;

        let mut response = self
            .db
            .query("SELECT * FROM $submission_id")
            .bind(("submission_id", submission_thing))
            .await?;

        let submissions: Vec<Submission> = response.take(0)?;
        submissions
            .into_iter()
            .next()
            .context(format!("Submission {} not found", submission_id))
    }

    pub async fn update_match_status(&self, match_id: &str, status: &str) -> Result<()> {
        let match_thing: Thing = match_id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_id))?;
        let query = if status == "running" {
            "UPDATE $match_id SET status = $status, started_at = time::now(), updated_at = time::now()"
        } else {
            "UPDATE $match_id SET status = $status, updated_at = time::now()"
        };

        self.db
            .query(query)
            .bind(("match_id", match_thing))
            .bind(("status", status.to_string()))
            .await?;

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
            if let Some(res) = result.participants.iter().find(|r| {
                match participant.submission_id.to_string().as_str() {
                    s => s == r.submission_id,
                }
            }) {
                participant.score = Some(res.score);
                participant.metadata = res.metadata.clone();
            }
        }

        self.db
            .query(
                "UPDATE $match_id SET
                    status = 'completed',
                    participants = $participants,
                    metadata = $metadata,
                    started_at = $started_at,
                    completed_at = $completed_at,
                    updated_at = time::now()",
            )
            .bind(("match_id", match_id.clone()))
            .bind(("participants", participants))
            .bind(("metadata", result.metadata))
            .bind(("started_at", started_at))
            .bind(("completed_at", completed_at))
            .await?;

        self.update_tournament_scores(match_data, &result.participants)
            .await?;

        Ok(())
    }

    pub async fn report_match_failure(&self, match_id: &str, error_msg: &str) -> Result<()> {
        let match_id: Thing = match_id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_id))?;

        self.db
            .query(
                "UPDATE $match_id SET
                    status = 'failed',
                    metadata = $metadata,
                    completed_at = time::now(),
                    updated_at = time::now()",
            )
            .bind(("match_id", match_id))
            .bind(("metadata", json!({ "error": error_msg })))
            .await?;

        Ok(())
    }

    pub async fn report_match_error(&self, match_id: &str, error_msg: &str) -> Result<()> {
        let match_id: Thing = match_id
            .parse()
            .map_err(|_| anyhow!("Invalid match id {}", match_id))?;

        self.db
            .query(
                "UPDATE $match_id SET
                    status = 'error',
                    metadata = $metadata,
                    completed_at = time::now(),
                    updated_at = time::now()",
            )
            .bind(("match_id", match_id))
            .bind(("metadata", json!({ "error": error_msg })))
            .await?;

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
    pub submission_id: Thing,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Game {
    pub id: Thing,
    #[serde(default)]
    pub game_code: Option<String>,
    #[serde(default)]
    pub game_language: Option<String>,
    #[serde(default)]
    pub turn_timeout_ms: Option<u64>,
    #[serde(default)]
    pub memory_limit_mb: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Submission {
    pub id: Thing,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
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
    pub metadata: Option<Value>,
}
