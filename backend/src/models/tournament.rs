use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: Option<Thing>,
    pub game_id: Thing, // Reference to Game
    pub name: String,
    pub description: String,
    pub status: TournamentStatus,
    pub min_players: u32,
    pub max_players: u32,
    pub current_players: u32,
    pub start_time: Option<Datetime>,
    pub end_time: Option<Datetime>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TournamentStatus {
    Scheduled,
    Registration,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentParticipant {
    pub id: Option<Thing>,
    pub tournament_id: Thing,
    pub user_id: Thing,
    pub submission_id: Option<Thing>, // Latest submission for this tournament
    pub score: f64,
    pub rank: Option<u32>,
    pub joined_at: Datetime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTournamentRequest {
    pub game_id: String, // Will be converted to Thing
    #[validate(length(
        min = 1,
        max = 100,
        message = "Tournament name must be 1-100 characters"
    ))]
    pub name: String,
    #[validate(length(min = 1, max = 1000, message = "Description must be 1-1000 characters"))]
    pub description: String,
    #[validate(range(min = 2, message = "Minimum players must be at least 2"))]
    pub min_players: u32,
    #[validate(range(min = 2, max = 500, message = "Maximum players must be 2-500"))]
    pub max_players: u32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTournamentRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Tournament name must be 1-100 characters"
    ))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 1000, message = "Description must be 1-1000 characters"))]
    pub description: Option<String>,
    pub status: Option<TournamentStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct JoinTournamentRequest {
    pub tournament_id: String,
}
