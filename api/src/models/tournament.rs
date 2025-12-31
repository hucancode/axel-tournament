use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: Option<Thing>,
    pub game_id: String, // Game ID (e.g., "rock-paper-scissors")
    pub name: String,
    pub description: String,
    pub status: TournamentStatus,
    pub min_players: u32,
    pub max_players: u32,
    pub start_time: Option<Datetime>,
    pub end_time: Option<Datetime>,
    pub match_generation_type: MatchGenerationType,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize)]
pub struct TournamentResponse {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub description: String,
    pub status: TournamentStatus,
    pub min_players: u32,
    pub max_players: u32,
    pub start_time: Option<Datetime>,
    pub end_time: Option<Datetime>,
    pub match_generation_type: MatchGenerationType,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<Tournament> for TournamentResponse {
    fn from(tournament: Tournament) -> Self {
        Self {
            id: tournament.id.map(|t| t.to_string()).unwrap_or_default(),
            game_id: tournament.game_id,
            name: tournament.name,
            description: tournament.description,
            status: tournament.status,
            min_players: tournament.min_players,
            max_players: tournament.max_players,
            start_time: tournament.start_time,
            end_time: tournament.end_time,
            match_generation_type: tournament.match_generation_type,
            created_at: tournament.created_at,
            updated_at: tournament.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchGenerationType {
    /// Each player plays against every other player (including themselves)
    /// For N players: N * N matches
    AllVsAll,
    /// Each player plays against every other player (excluding themselves)
    /// For N players: N * (N-1) matches
    RoundRobin,
    /// Single elimination bracket
    SingleElimination,
    /// Double elimination bracket
    DoubleElimination,
}

impl Default for MatchGenerationType {
    fn default() -> Self {
        MatchGenerationType::AllVsAll
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TournamentStatus {
    Scheduled,
    Registration,
    Generating,
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
    pub match_generation_type: Option<MatchGenerationType>, // Defaults to AllVsAll if not provided
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
