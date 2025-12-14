use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user_id: String,
    pub username: String,
    pub location: String,
    pub score: f64,
    pub tournament_name: String,
    pub tournament_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<u32>, // Top K players
    pub tournament_id: Option<String>,
    pub game_id: Option<String>,
}
