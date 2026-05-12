use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Debug, Clone, Deserialize, SurrealValue)]
pub struct Tournament {
    pub id: Option<RecordId>,
    pub game_id: String,
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
    /// Free-form per-game configuration. Copied to every ranked room
    /// spawned by matchmaking; the judge logic owns the schema.
    #[serde(default = "default_config")]
    pub config: Value,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

fn default_config() -> Value {
    Value::Object(serde_json::Map::new())
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
    /// Continuous score-pairing (no bracket). Players queue and get
    /// paired by closest score; matches keep flowing until the
    /// tournament's `end_time` is reached.
    #[surreal(rename = "continuous")]
    Continuous,
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
    pub submission_id: Option<RecordId>,
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
