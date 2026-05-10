// ELO math + persistence on tournament_participant.
//
// Two scoring modes:
//   * Elo  — classic 1v1 win/lose ELO update (chess, xiangqi, tic-tac-toe).
//   * Score — accumulate per-player score into the same `elo` column
//     (rps, prisoners-dilemma, poker). Score games use the column as a
//     running total rather than a rating; the leaderboard ranks by the
//     same field either way. Each finished match adds the winner-share
//     (1.0 / 0.5 / 0.0) to the score; poker would replace this with chip
//     totals once the judge → api score channel is wired up.

use crate::{db::Database, error::ApiResult, models::game::ScoringKind};
use surrealdb::types::RecordId;

pub const DEFAULT_ELO: f64 = 1000.0;
pub const ELO_K: f64 = 32.0;
pub const SCORE_SEED: f64 = 1000.0;

pub fn update_elo_pair(elo_a: f64, elo_b: f64, sa: f64, sb: f64) -> (f64, f64) {
    let ea = expected_score(elo_a, elo_b);
    let eb = 1.0 - ea;
    let na = elo_a + ELO_K * (sa - ea);
    let nb = elo_b + ELO_K * (sb - eb);
    (na, nb)
}

fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// Apply ranked-match adjustment to participants. Branches on the
/// game's `scoring_kind`:
///   * Elo   — classic K=32 adjustment with seed 1000.
///   * Score — add raw per-player score deltas (e.g. poker chip total
///             minus starting stack) to participant's score column. If
///             `scores` is absent, fall back to winner-share (1.0/0.5/0.0).
/// No-op for non-1v1 player counts.
pub async fn apply_ranked_result(
    db: &Database,
    tournament_id: &RecordId,
    game_id: &str,
    players: &[RecordId],
    winner: &Option<RecordId>,
    scores: Option<&[f64]>,
) -> ApiResult<()> {
    if players.len() != 2 {
        return Ok(());
    }
    let a = &players[0];
    let b = &players[1];

    let kind = crate::models::game::find_game_by_id(game_id)
        .map(|g| g.scoring_kind)
        .unwrap_or(ScoringKind::Elo);

    let (sa, sb) = match winner {
        Some(w) if w == a => (1.0, 0.0),
        Some(w) if w == b => (0.0, 1.0),
        _ => (0.5, 0.5),
    };

    match kind {
        ScoringKind::Elo => {
            let elo_a = current_score(db, tournament_id, a, DEFAULT_ELO).await?;
            let elo_b = current_score(db, tournament_id, b, DEFAULT_ELO).await?;
            let (na, nb) = update_elo_pair(elo_a, elo_b, sa, sb);
            write_score(db, tournament_id, a, na).await?;
            write_score(db, tournament_id, b, nb).await?;
        }
        ScoringKind::Score => {
            // Use raw chip/round-score deltas if the judge supplied them
            // via match.participants[].score; otherwise fall back to the
            // 1.0/0.5/0.0 winner share.
            let (da, db_) = match scores {
                Some(s) if s.len() >= 2 => (s[0], s[1]),
                _ => (sa, sb),
            };
            let score_a = current_score(db, tournament_id, a, SCORE_SEED).await?;
            let score_b = current_score(db, tournament_id, b, SCORE_SEED).await?;
            write_score(db, tournament_id, a, score_a + da).await?;
            write_score(db, tournament_id, b, score_b + db_).await?;
        }
    }
    Ok(())
}

async fn current_score(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
    default: f64,
) -> ApiResult<f64> {
    Ok(crate::services::tournament::get_participant(db, tournament_id, user_id)
        .await?
        .and_then(|p| p.elo)
        .unwrap_or(default))
}

async fn write_score(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
    score: f64,
) -> ApiResult<()> {
    db.query(
        "UPDATE tournament_participant SET elo = $elo
         WHERE tournament_id = $tid AND user_id = $uid",
    )
    .bind(("tid", tournament_id.clone()))
    .bind(("uid", user_id.clone()))
    .bind(("elo", score))
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_elo_winner_gains_loser_drops() {
        let (na, nb) = update_elo_pair(1000.0, 1000.0, 1.0, 0.0);
        assert!((na - 1016.0).abs() < 0.01);
        assert!((nb - 984.0).abs() < 0.01);
    }

    #[test]
    fn draw_between_equal_changes_nothing() {
        let (na, nb) = update_elo_pair(1000.0, 1000.0, 0.5, 0.5);
        assert!((na - 1000.0).abs() < 0.01);
        assert!((nb - 1000.0).abs() < 0.01);
    }

    #[test]
    fn upset_costs_favourite_more_than_routine_win() {
        let (a_routine, _) = update_elo_pair(1300.0, 1000.0, 1.0, 0.0);
        let (a_upset, b_upset) = update_elo_pair(1000.0, 1300.0, 1.0, 0.0);
        assert!(a_routine - 1300.0 < a_upset - 1000.0);
        assert!(b_upset < 1300.0);
    }
}
