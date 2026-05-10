// Read-side aggregates over match rows. Pure DB queries; no writes.

use crate::{
    db::Database,
    error::ApiResult,
    models::matches::{Match, MatchStatus},
};
use surrealdb::types::{RecordId, ToSql};

/// Aggregate statistics for one submission's bot across every match it
/// played. `wins`/`losses`/`draws` are decided by score comparison (the
/// match's other participant). `total_score` is the absolute sum so
/// scored games (e.g. PD payoff) still surface a meaningful value.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubmissionStats {
    pub submission_id: String,
    pub matches_played: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total_score: f64,
}

pub async fn submission_stats(
    db: &Database,
    submission_id: RecordId,
) -> ApiResult<SubmissionStats> {
    let mut resp = db
        .query(
            "SELECT * FROM match
             WHERE participants[*].submission_id CONTAINS $sid
             AND status IN ['completed', 'failed']",
        )
        .bind(("sid", submission_id.clone()))
        .await?;
    let matches: Vec<Match> = resp.take(0)?;

    let sid_sql = submission_id.to_sql();
    let mut stats = SubmissionStats {
        submission_id: sid_sql.clone(),
        matches_played: 0,
        wins: 0,
        losses: 0,
        draws: 0,
        total_score: 0.0,
    };

    for m in matches {
        let me_idx = m.participants.iter().position(|p| {
            p.submission_id
                .as_ref()
                .is_some_and(|s| s.to_sql() == sid_sql)
        });
        let Some(me_idx) = me_idx else { continue };
        stats.matches_played += 1;

        if m.status == MatchStatus::Failed {
            stats.losses += 1;
            continue;
        }
        let my_score = m.participants[me_idx].score.unwrap_or(0.0);
        stats.total_score += my_score;

        let other_max = m
            .participants
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != me_idx)
            .filter_map(|(_, p)| p.score)
            .fold(f64::NEG_INFINITY, f64::max);
        if !other_max.is_finite() {
            continue;
        }
        if my_score > other_max {
            stats.wins += 1;
        } else if my_score < other_max {
            stats.losses += 1;
        } else {
            stats.draws += 1;
        }
    }
    Ok(stats)
}
