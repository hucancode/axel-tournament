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

/// Helper function to parse simple space-separated match results
/// Format: "100 85" or "TLE 92" or "100 WA"
pub fn parse_simple_result(output: &str, submission_ids: &[String]) -> GameResult<MatchExecutionResult> {
    let parts: Vec<&str> = output.trim().split_whitespace().collect();

    if parts.len() != submission_ids.len() {
        return Err(crate::GameError::Other(
            format!("Expected {} results, got {}", submission_ids.len(), parts.len())
        ));
    }

    let participant_results = parts.iter().zip(submission_ids.iter()).map(|(part, sub_id)| {
        let (score, error_code) = match part.parse::<i32>() {
            Ok(s) => (s, None),
            Err(_) => {
                // This is an error code like TLE, WA, RE, CE
                (0, Some(part.to_string()))
            }
        };

        crate::ParticipantResult {
            submission_id: sub_id.clone(),
            score,
            error_code,
        }
    }).collect();

    Ok(MatchExecutionResult {
        participant_results,
        metadata: serde_json::json!({}),
    })
}
