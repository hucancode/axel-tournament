use serde::{Deserialize, Serialize};
use surrealdb::types::Datetime;
#[cfg(test)]
use surrealdb::types::RecordId;
use validator::Validate;

pub use axel_core::models::matches::{Match, MatchParticipant, MatchStatus};

use super::{bare_key, opt_bare_key, vec_bare_key};

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
