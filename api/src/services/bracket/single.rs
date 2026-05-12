use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{matches::Match, tournament::Tournament},
};

use super::common::write_bracket_match;

/// Single-elim advancement: pair winners of round N's positions 2k and
/// 2k+1 into round N+1 position k. BYE matches (one participant) skip
/// to next round automatically.
pub(super) async fn advance_single(
    db: &Database,
    tournament: &Tournament,
    matches: &[Match],
) -> AppResult<u32> {
    let mut created = 0;
    let max_round = matches
        .iter()
        .filter_map(|m| m.round)
        .max()
        .unwrap_or(0);

    let next_round = max_round + 1;
    let current: Vec<&Match> = matches
        .iter()
        .filter(|m| m.round == Some(max_round) && m.bracket.as_deref() == Some("winners"))
        .collect();

    // Already moved on?
    if matches
        .iter()
        .any(|m| m.round == Some(next_round) && m.bracket.as_deref() == Some("winners"))
    {
        return Ok(0);
    }
    // Some current match still pending? Wait.
    if current.iter().any(|m| !m.status.is_terminal()) {
        return Ok(0);
    }
    if current.len() < 2 {
        // Round of one = bracket finished. Caller's finalize flips the
        // tournament when all matches are terminal.
        return Ok(0);
    }
    // Pair current matches by position.
    let mut by_pos: Vec<(u32, &Match)> = current
        .iter()
        .map(|m| (m.bracket_position.unwrap_or(0), *m))
        .collect();
    by_pos.sort_by_key(|(p, _)| *p);
    for chunk in by_pos.chunks(2) {
        if chunk.len() < 2 {
            break;
        }
        let (pa, ma) = chunk[0];
        let (_, mb) = chunk[1];
        let wa = ma
            .winner()
            .ok_or_else(|| AppError::Internal("bracket advance: missing winner".to_string()))?;
        let wb = mb
            .winner()
            .ok_or_else(|| AppError::Internal("bracket advance: missing winner".to_string()))?;
        write_bracket_match(
            db,
            tournament,
            wa,
            Some(wb),
            next_round,
            "winners",
            pa / 2,
        )
        .await?;
        created += 1;
    }
    Ok(created)
}
