use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::types::Datetime;
use validator::Validate;

pub use axel_core::models::tournament::{
    MatchGenerationType, Tournament, TournamentKind, TournamentParticipant, TournamentStatus,
};

use super::{bare_key, opt_bare_key};

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
    pub config: Value,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    #[serde(default)]
    pub participant_count: u32,
}

impl From<Tournament> for TournamentResponse {
    fn from(t: Tournament) -> Self {
        Self::from((t, 0u32))
    }
}

impl From<(Tournament, u32)> for TournamentResponse {
    fn from((t, participant_count): (Tournament, u32)) -> Self {
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
            config: t.config,
            created_at: t.created_at,
            updated_at: t.updated_at,
            participant_count,
        }
    }
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
    pub username: Option<String>,
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
            username: None,
        }
    }
}

impl From<(TournamentParticipant, Option<String>)> for TournamentParticipantResponse {
    fn from((p, username): (TournamentParticipant, Option<String>)) -> Self {
        let mut resp: TournamentParticipantResponse = p.into();
        resp.username = username;
        resp
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTournamentRequest {
    pub game_id: String,
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
    pub match_generation_type: Option<MatchGenerationType>,
    pub kind: Option<TournamentKind>,
    /// Free-form per-game configuration (board size, rounds, time, blind
    /// mode, etc.). Each game's logic owns the schema; api passes through.
    pub config: Option<Value>,
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
