use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};
use validator::Validate;

use super::{bare_key, opt_bare_key, vec_bare_key};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
pub struct Match {
    pub id: Option<RecordId>,
    pub tournament_id: Option<RecordId>, // Optional for standalone interactive matches
    pub game_id: String, // Changed from Thing - games are now hardcoded
    pub status: MatchStatus,
    pub participants: Vec<MatchParticipant>,
    pub metadata: Option<serde_json::Value>, // For game-specific replay data or logs
    pub room_id: Option<RecordId>, // For interactive matches
    pub game_event_source: Option<String>, // Game state history for reconnection
    pub judge_server_name: Option<String>, // Which judge server claimed this match
    #[serde(default)]
    pub error_message: Option<String>, // Set when match transitions to Failed
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

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MatchParticipant {
    pub user_id: RecordId,
    pub submission_id: Option<RecordId>, // For automated matches
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchResponse {
    pub id: Option<String>,
    pub tournament_id: Option<String>,
    pub game_id: String,
    pub status: MatchStatus,
    pub participants: Vec<MatchParticipantResponse>,
    pub metadata: Option<serde_json::Value>,
    pub room_id: Option<String>,
    pub game_event_source: Option<String>,
    pub judge_server_name: Option<String>,
    pub error_message: Option<String>,
    pub faulted_user_ids: Vec<String>,
    pub round: Option<u32>,
    pub bracket: Option<String>,
    pub bracket_position: Option<u32>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub started_at: Option<Datetime>,
    pub completed_at: Option<Datetime>,
}

impl From<Match> for MatchResponse {
    fn from(m: Match) -> Self {
        Self {
            id: opt_bare_key(&m.id),
            tournament_id: opt_bare_key(&m.tournament_id),
            game_id: m.game_id,
            status: m.status,
            participants: m.participants.into_iter().map(Into::into).collect(),
            metadata: m.metadata,
            room_id: opt_bare_key(&m.room_id),
            game_event_source: m.game_event_source,
            judge_server_name: m.judge_server_name,
            error_message: m.error_message,
            faulted_user_ids: vec_bare_key(&m.faulted_user_ids),
            round: m.round,
            bracket: m.bracket,
            bracket_position: m.bracket_position,
            created_at: m.created_at,
            updated_at: m.updated_at,
            started_at: m.started_at,
            completed_at: m.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchParticipantResponse {
    pub user_id: String,
    pub submission_id: Option<String>,
    pub score: Option<f64>,
}

impl From<MatchParticipant> for MatchParticipantResponse {
    fn from(p: MatchParticipant) -> Self {
        Self {
            user_id: bare_key(&p.user_id),
            submission_id: opt_bare_key(&p.submission_id),
            score: p.score,
        }
    }
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMatchRequest {
    pub tournament_id: String,
    pub game_id: String,

    #[validate(length(min = 2, message = "Match must have at least 2 participants"))]
    pub participant_submission_ids: Vec<String>,
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub fn user(s: &str) -> RecordId {
        RecordId::parse_simple(&format!("user:{s}")).unwrap()
    }

    pub fn part(uid: &str, score: Option<f64>) -> MatchParticipant {
        MatchParticipant {
            user_id: user(uid),
            submission_id: None,
            score,
        }
    }

    pub fn finished(p1: &str, s1: f64, p2: &str, s2: f64) -> Match {
        Match {
            game_id: "g".into(),
            status: MatchStatus::Completed,
            participants: vec![part(p1, Some(s1)), part(p2, Some(s2))],
            ..Default::default()
        }
    }

    pub fn failed(p1: &str, p2: &str) -> Match {
        Match {
            status: MatchStatus::Failed,
            error_message: Some("compile_error".into()),
            ..finished(p1, 0.0, p2, 0.0)
        }
    }

    pub fn faulted(p1: &str, p2: &str, faulted_user: &str) -> Match {
        Match {
            status: MatchStatus::Failed,
            error_message: Some("runtime_error".into()),
            faulted_user_ids: vec![user(faulted_user)],
            ..finished(p1, 0.0, p2, 0.0)
        }
    }

    pub fn pending(p1: &str, p2: &str) -> Match {
        Match {
            status: MatchStatus::Pending,
            ..finished(p1, 0.0, p2, 0.0)
        }
    }
}
