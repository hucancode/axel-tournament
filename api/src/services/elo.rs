// ELO math + persistence on tournament_participant.

use crate::{db::Database, error::ApiResult};
use surrealdb::types::RecordId;

pub const DEFAULT_ELO: f64 = 1000.0;
pub const ELO_K: f64 = 32.0;

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

/// Apply ELO-style adjustment to participants based on a 1v1 result.
/// Default starting ELO 1000, K-factor 32. Draws split 0.5/0.5. No-op
/// for non-1v1 player counts.
pub async fn apply_ranked_result(
    db: &Database,
    tournament_id: &RecordId,
    players: &[RecordId],
    winner: &Option<RecordId>,
) -> ApiResult<()> {
    if players.len() != 2 {
        return Ok(());
    }
    let a = &players[0];
    let b = &players[1];

    let elo_a = current_elo(db, tournament_id, a).await?;
    let elo_b = current_elo(db, tournament_id, b).await?;

    let (sa, sb) = match winner {
        Some(w) if w == a => (1.0, 0.0),
        Some(w) if w == b => (0.0, 1.0),
        _ => (0.5, 0.5),
    };
    let (na, nb) = update_elo_pair(elo_a, elo_b, sa, sb);
    write_elo(db, tournament_id, a, na).await?;
    write_elo(db, tournament_id, b, nb).await?;
    Ok(())
}

async fn current_elo(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
) -> ApiResult<f64> {
    Ok(crate::services::tournament::get_participant(db, tournament_id, user_id)
        .await?
        .and_then(|p| p.elo)
        .unwrap_or(DEFAULT_ELO))
}

async fn write_elo(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
    elo: f64,
) -> ApiResult<()> {
    db.query(
        "UPDATE tournament_participant SET elo = $elo
         WHERE tournament_id = $tid AND user_id = $uid",
    )
    .bind(("tid", tournament_id.clone()))
    .bind(("uid", user_id.clone()))
    .bind(("elo", elo))
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
