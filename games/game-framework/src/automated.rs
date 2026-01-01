use crate::db_client::MatchParticipant;
use crate::{GameResult, MatchExecutionResult};

/// Trait for automated game servers that execute bot vs bot matches
pub trait AutomatedGameServer {
    /// Execute a match between automated players
    ///
    /// # Arguments
    /// * `player_binaries` - Paths to compiled player executables
    /// * `rounds` - Number of rounds to play
    /// * `turn_timeout_ms` - Timeout per turn in milliseconds
    ///
    /// # Returns
    /// Match results with scores for each player
    fn execute_match(
        &self,
        player_binaries: Vec<String>,
        rounds: u32,
        turn_timeout_ms: u64,
    ) -> GameResult<MatchExecutionResult>;
}

/// Helper function to format match results as space-separated string
/// Format: "100 85" or "TLE 92" or "100 WA"
///
/// # Examples
/// ```
/// use game_framework::{format_simple_result, ParticipantResult};
/// use surrealdb::sql::Thing;
///
/// // Two players with scores
/// let results = vec![
///     ParticipantResult {
///         submission_id: Thing::from(("submission".to_string(), "1".to_string())),
///         user_id: Thing::from(("user".to_string(), "1".to_string())),
///         score: 100,
///         error_code: None,
///     },
///     ParticipantResult {
///         submission_id: Thing::from(("submission".to_string(), "2".to_string())),
///         user_id: Thing::from(("user".to_string(), "2".to_string())),
///         score: 85,
///         error_code: None,
///     },
/// ];
/// assert_eq!(format_simple_result(&results), "100 85");
///
/// // Player 1 timeout, Player 2 success
/// let results = vec![
///     ParticipantResult {
///         submission_id: Thing::from(("submission".to_string(), "1".to_string())),
///         user_id: Thing::from(("user".to_string(), "1".to_string())),
///         score: 0,
///         error_code: Some("TLE".to_string()),
///     },
///     ParticipantResult {
///         submission_id: Thing::from(("submission".to_string(), "2".to_string())),
///         user_id: Thing::from(("user".to_string(), "2".to_string())),
///         score: 92,
///         error_code: None,
///     },
/// ];
/// assert_eq!(format_simple_result(&results), "TLE 92");
/// ```
pub fn format_simple_result(participant_results: &[crate::ParticipantResult]) -> String {
    participant_results
        .iter()
        .map(|result| {
            if let Some(ref error_code) = result.error_code {
                error_code.clone()
            } else {
                result.score.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Helper function to parse simple space-separated match results
/// Format: "100 85" or "TLE 92" or "100 WA"
pub fn parse_simple_result(
    output: &str,
    participants: &[MatchParticipant],
) -> GameResult<MatchExecutionResult> {
    let parts: Vec<&str> = output.trim().split_whitespace().collect();

    if parts.len() != participants.len() {
        return Err(crate::GameError::Other(format!(
            "Expected {} results, got {}",
            participants.len(),
            parts.len()
        )));
    }

    let participant_results = parts
        .iter()
        .zip(participants.iter())
        .map(|(part, participant)| {
            let (score, error_code) = match part.parse::<i32>() {
                Ok(s) => (s, None),
                Err(_) => {
                    // This is an error code like TLE, WA, RE
                    (0, Some(part.to_string()))
                }
            };

            crate::ParticipantResult {
                submission_id: participant.submission_id.clone(),
                user_id: participant.user_id.clone(),
                score,
                error_code,
            }
        })
        .collect();

    Ok(MatchExecutionResult {
        participant_results,
        metadata: serde_json::json!({}),
    })
}
