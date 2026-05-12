use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

/// Canonical match row. Single source of truth for api + core. Judge
/// keeps a trimmed bot-match-specific shape (`models::match_row`) where
/// `submission_id` is required by invariant.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
pub struct Match {
    pub id: Option<RecordId>,
    /// None for standalone interactive matches (playground/room).
    pub tournament_id: Option<RecordId>,
    pub game_id: String,
    pub status: MatchStatus,
    pub participants: Vec<MatchParticipant>,
    /// Game-specific replay data or logs.
    pub metadata: Option<serde_json::Value>,
    /// Set for interactive matches.
    pub room_id: Option<RecordId>,
    /// Game state history for reconnection.
    pub game_event_source: Option<String>,
    /// Which judge server claimed this match.
    pub judge_server_name: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    /// Users at fault (runtime error, illegal move, compile fail) for
    /// this match. Empty when nobody is at fault — typical Completed.
    /// Per-participant losses come from this list, not the whole-match
    /// `Failed` status.
    #[serde(default)]
    pub faulted_user_ids: Vec<RecordId>,
    /// Bracket-only metadata. `round` 0 = first round; `bracket` is
    /// "winners" / "losers" / "grand_final" for double-elim. NONE for
    /// round-robin / all-vs-all matches.
    #[serde(default)]
    pub round: Option<u32>,
    #[serde(default)]
    pub bracket: Option<String>,
    /// Position within the round (0-indexed). Used to wire the winner
    /// of this match into participant slot of the next round's match.
    #[serde(default)]
    pub bracket_position: Option<u32>,
    /// Set to true once `apply_ranked_result` has consumed this row,
    /// so the healer's ELO/score-accumulator pass is idempotent. Only
    /// meaningful for ranked matches.
    #[serde(default)]
    pub elo_applied: bool,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub started_at: Option<Datetime>,
    pub completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue, Default)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum MatchStatus {
    #[default]
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl MatchStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, MatchStatus::Completed | MatchStatus::Failed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MatchParticipant {
    pub user_id: RecordId,
    /// None for live human matches.
    pub submission_id: Option<RecordId>,
    #[serde(default)]
    pub score: Option<f64>,
}

impl Match {
    /// Single bracket-style winner (or `None` if undecidable).
    /// Completed: top score among non-faulted participants. Failed:
    /// only if exactly one non-faulted participant remains. Other
    /// statuses: `None`.
    pub fn winner(&self) -> Option<MatchParticipant> {
        match self.status {
            MatchStatus::Completed => {
                let mut best: Option<&MatchParticipant> = None;
                for p in &self.participants {
                    if self.faulted_user_ids.contains(&p.user_id) {
                        continue;
                    }
                    let s = p.score.unwrap_or(0.0);
                    match best {
                        None => best = Some(p),
                        Some(b) if s > b.score.unwrap_or(0.0) => best = Some(p),
                        _ => {}
                    }
                }
                best.cloned()
            }
            MatchStatus::Failed => {
                let alive: Vec<&MatchParticipant> = self
                    .participants
                    .iter()
                    .filter(|p| !self.faulted_user_ids.contains(&p.user_id))
                    .collect();
                (alive.len() == 1).then(|| alive[0].clone())
            }
            _ => None,
        }
    }
}
