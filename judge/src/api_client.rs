use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Match {
    pub id: String,
    pub game_id: String,
    pub tournament_id: Option<String>,
    pub participants: Vec<MatchParticipant>,
}

#[derive(Debug, Deserialize)]
pub struct MatchParticipant {
    pub submission_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct MatchResult {
    pub status: String,
    pub participants: Vec<ParticipantResult>,
    pub metadata: serde_json::Value,
    pub started_at: String,
    pub completed_at: String,
}

#[derive(Debug, Serialize)]
pub struct ParticipantResult {
    pub submission_id: String,
    pub score: f64,
    pub rank: Option<u32>,
    pub is_winner: bool,
    pub metadata: Option<serde_json::Value>,
}

impl ApiClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn fetch_pending_matches(&self) -> Result<Vec<Match>> {
        let url = format!("{}/api/matches?status=pending", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn update_match_status(&self, match_id: &str, status: &str) -> Result<()> {
        let url = format!("{}/api/matches/{}/result", self.base_url, match_id);
        let payload = serde_json::json!({
            "status": status,
            "participants": []
        });

        self.client
            .put(&url)
            .header("X-Match-Runner-Key", &self.api_key)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }

    pub async fn report_match_result(&self, match_id: &str, result: MatchResult) -> Result<()> {
        let url = format!("{}/api/matches/{}/result", self.base_url, match_id);

        self.client
            .put(&url)
            .header("X-Match-Runner-Key", &self.api_key)
            .json(&result)
            .send()
            .await?;

        Ok(())
    }

    pub async fn report_match_failure(&self, match_id: &str, error_msg: &str) -> Result<()> {
        let url = format!("{}/api/matches/{}/result", self.base_url, match_id);
        let payload = serde_json::json!({
            "status": "failed",
            "participants": [],
            "metadata": {
                "error": error_msg
            }
        });

        self.client
            .put(&url)
            .header("X-Match-Runner-Key", &self.api_key)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }
}
