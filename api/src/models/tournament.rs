use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};
use validator::Validate;

use super::{bare_key, opt_bare_key};

#[derive(Debug, Clone, Deserialize, SurrealValue)]
pub struct Tournament {
    pub id: Option<RecordId>,
    pub game_id: String, // Game ID (e.g., "rock-paper-scissors")
    pub name: String,
    pub description: String,
    pub status: TournamentStatus,
    pub min_players: u32,
    pub max_players: u32,
    pub start_time: Option<Datetime>,
    pub end_time: Option<Datetime>,
    pub match_generation_type: MatchGenerationType,
    #[serde(default)]
    pub kind: TournamentKind,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize)]
pub struct TournamentResponse {
    pub id: Option<String>,
    pub game_id: String,
    pub name: String,
    pub description: String,
    pub status: TournamentStatus,
    pub min_players: u32,
    pub max_players: u32,
    pub start_time: Option<Datetime>,
    pub end_time: Option<Datetime>,
    pub match_generation_type: MatchGenerationType,
    pub kind: TournamentKind,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<Tournament> for TournamentResponse {
    fn from(t: Tournament) -> Self {
        Self {
            id: opt_bare_key(&t.id),
            game_id: t.game_id,
            name: t.name,
            description: t.description,
            status: t.status,
            min_players: t.min_players,
            max_players: t.max_players,
            start_time: t.start_time,
            end_time: t.end_time,
            match_generation_type: t.match_generation_type,
            kind: t.kind,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum TournamentKind {
    /// Code-vs-code: bots upload, judge runs matches.
    Bot,
    /// Human-vs-human: players play through rooms.
    Human,
}

impl Default for TournamentKind {
    fn default() -> Self {
        Self::Bot
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "snake_case")]
#[surreal(untagged)]
pub enum MatchGenerationType {
    /// Each player plays against every other player (including themselves)
    /// For N players: N * N matches
    #[surreal(rename = "all_vs_all")]
    AllVsAll,
    /// Each player plays against every other player (excluding themselves)
    /// For N players: N * (N-1) matches
    #[surreal(rename = "round_robin")]
    RoundRobin,
    /// Single elimination bracket
    #[surreal(rename = "single_elimination")]
    SingleElimination,
    /// Double elimination bracket
    #[surreal(rename = "double_elimination")]
    DoubleElimination,
}

impl Default for MatchGenerationType {
    fn default() -> Self {
        MatchGenerationType::AllVsAll
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum TournamentStatus {
    Scheduled,
    Registration,
    Generating,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Deserialize, SurrealValue)]
pub struct TournamentParticipant {
    pub id: Option<RecordId>,
    pub tournament_id: RecordId,
    pub user_id: RecordId,
    pub submission_id: Option<RecordId>, // Active submission for this tournament
    pub score: f64,
    #[serde(default)]
    pub wins: u32,
    #[serde(default)]
    pub losses: u32,
    #[serde(default)]
    pub draws: u32,
    #[serde(default)]
    pub elo: Option<f64>,
    pub rank: Option<u32>,
    pub joined_at: Datetime,
}

#[derive(Debug, Clone, Serialize)]
pub struct TournamentParticipantResponse {
    pub id: Option<String>,
    pub tournament_id: String,
    pub user_id: String,
    pub submission_id: Option<String>,
    pub score: f64,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub elo: Option<f64>,
    pub rank: Option<u32>,
    pub joined_at: Datetime,
}

impl From<TournamentParticipant> for TournamentParticipantResponse {
    fn from(p: TournamentParticipant) -> Self {
        Self {
            id: opt_bare_key(&p.id),
            tournament_id: bare_key(&p.tournament_id),
            user_id: bare_key(&p.user_id),
            submission_id: opt_bare_key(&p.submission_id),
            score: p.score,
            wins: p.wins,
            losses: p.losses,
            draws: p.draws,
            elo: p.elo,
            rank: p.rank,
            joined_at: p.joined_at,
        }
    }
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
    pub kind: Option<TournamentKind>, // Defaults to Bot if not provided
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
