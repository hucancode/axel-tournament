use crate::{
    db::Database,
    error::AppResult,
    models::{matches::{Match, MatchParticipant}, tournament::Tournament},
};

use super::common::write_bracket_match;
use super::single::advance_single;

/// Double-elim advancement.
///
/// Layout (power-of-2 N, k = log2(N)):
///   * Winners bracket: WB R0..WB R(k-1), standard single-elim.
///   * Losers bracket: LB R0..LB R(2k-3), 2(k-1) rounds.
///       - LB R0           : pairs WB R0 losers.
///       - LB R(2j+1) drop : LB R(2j) winners + WB R(j+1) losers.
///       - LB R(2j+2) inner: pairs LB R(2j+1) winners.
///   * Grand final         : WB final winner vs LB final winner.
///   * Grand final reset   : created only when LB winner wins GF.
pub(super) async fn advance_double(
    db: &Database,
    tournament: &Tournament,
    matches: &[Match],
) -> AppResult<u32> {
    let mut created = 0u32;
    created += advance_single(db, tournament, matches).await?;
    created += advance_losers(db, tournament, matches).await?;
    created += advance_grand_final(db, tournament, matches).await?;
    Ok(created)
}

/// log2 of WB R0 count, rounded up. For exact power-of-2 fields this
/// is k. For non-power-of-2 the WB still pads with byes; LB is more
/// awkward and we do not generate it here.
fn k_factor(matches: &[Match]) -> u32 {
    let r0 = matches
        .iter()
        .filter(|m| m.bracket.as_deref() == Some("winners") && m.round == Some(0))
        .count() as u32;
    // r0 = N/2 for full bracket. k = log2(N) = log2(r0) + 1.
    let mut k = 0u32;
    let mut x = r0.max(1);
    while x > 1 {
        x /= 2;
        k += 1;
    }
    k + 1
}

async fn advance_losers(
    db: &Database,
    tournament: &Tournament,
    matches: &[Match],
) -> AppResult<u32> {
    let k = k_factor(matches);
    if k < 2 {
        return Ok(0); // Trivial bracket; nothing to drop.
    }
    let total_lb_rounds = 2 * (k - 1);
    let mut created = 0u32;

    for lb_round in 0..total_lb_rounds {
        if matches
            .iter()
            .any(|m| m.bracket.as_deref() == Some("losers") && m.round == Some(lb_round))
        {
            continue; // Already generated this round.
        }
        let pairs = lb_round_pairs(matches, lb_round);
        let Some(pairs) = pairs else {
            // Inputs not yet available (a feeder match still pending).
            break;
        };
        for (pos, (a, b)) in pairs.into_iter().enumerate() {
            write_bracket_match(
                db,
                tournament,
                a,
                Some(b),
                lb_round,
                "losers",
                pos as u32,
            )
            .await?;
            created += 1;
        }
    }
    Ok(created)
}

/// Returns the list of `(player_a, player_b)` for every match in LB
/// round `r`, or `None` if any feeder match is not yet terminal.
fn lb_round_pairs(
    matches: &[Match],
    r: u32,
) -> Option<Vec<(MatchParticipant, MatchParticipant)>> {
    if r == 0 {
        // Pair WB R0 losers position 2p with 2p+1.
        let losers = round_losers(matches, "winners", 0)?;
        let mut pairs = Vec::with_capacity(losers.len() / 2);
        for chunk in losers.chunks(2) {
            if chunk.len() < 2 {
                break;
            }
            pairs.push((chunk[0].clone(), chunk[1].clone()));
        }
        return Some(pairs);
    }
    if r % 2 == 1 {
        // Drop round LB R(2j+1) <- LB R(2j) winners + WB R(j+1) losers.
        let j = (r - 1) / 2;
        let lb_winners = round_winners(matches, "losers", 2 * j)?;
        let wb_losers = round_losers(matches, "winners", j + 1)?;
        if lb_winners.len() != wb_losers.len() {
            return None;
        }
        let mut pairs = Vec::with_capacity(lb_winners.len());
        for (a, b) in lb_winners.into_iter().zip(wb_losers.into_iter()) {
            pairs.push((a, b));
        }
        return Some(pairs);
    }
    // r is even and > 0: internal round LB R(2j+2) pairs LB R(2j+1) winners.
    let j = r / 2 - 1;
    let lb_winners = round_winners(matches, "losers", 2 * j + 1)?;
    let mut pairs = Vec::with_capacity(lb_winners.len() / 2);
    for chunk in lb_winners.chunks(2) {
        if chunk.len() < 2 {
            break;
        }
        pairs.push((chunk[0].clone(), chunk[1].clone()));
    }
    Some(pairs)
}

fn round_outcomes(
    matches: &[Match],
    bracket: &str,
    round: u32,
    pick: impl Fn(&Match) -> Option<MatchParticipant>,
) -> Option<Vec<MatchParticipant>> {
    let mut rows: Vec<&Match> = matches
        .iter()
        .filter(|m| m.bracket.as_deref() == Some(bracket) && m.round == Some(round))
        .collect();
    if rows.is_empty() {
        return None;
    }
    rows.sort_by_key(|m| m.bracket_position.unwrap_or(0));
    rows.into_iter().map(pick).collect()
}

fn round_winners(
    matches: &[Match],
    bracket: &str,
    round: u32,
) -> Option<Vec<MatchParticipant>> {
    round_outcomes(matches, bracket, round, Match::winner)
}

fn round_losers(
    matches: &[Match],
    bracket: &str,
    round: u32,
) -> Option<Vec<MatchParticipant>> {
    round_outcomes(matches, bracket, round, loser_of)
}

fn loser_of(m: &Match) -> Option<MatchParticipant> {
    if !m.status.is_terminal() {
        return None;
    }
    let winner = m.winner()?;
    m.participants
        .iter()
        .find(|p| p.user_id != winner.user_id)
        .cloned()
}

async fn advance_grand_final(
    db: &Database,
    tournament: &Tournament,
    matches: &[Match],
) -> AppResult<u32> {
    let k = k_factor(matches);
    if k < 1 {
        return Ok(0);
    }

    // Grand final exists yet?
    let gf = matches
        .iter()
        .find(|m| m.bracket.as_deref() == Some("grand_final"));
    let reset = matches
        .iter()
        .find(|m| m.bracket.as_deref() == Some("grand_final_reset"));

    if gf.is_none() {
        // Need WB final winner + LB final winner.
        let wb_final_round = k - 1;
        let lb_final_round = if k >= 2 { 2 * (k - 1) - 1 } else { return Ok(0) };
        let wb_winner = round_winners(matches, "winners", wb_final_round)
            .and_then(|v| v.into_iter().next());
        let lb_winner = round_winners(matches, "losers", lb_final_round)
            .and_then(|v| v.into_iter().next());
        match (wb_winner, lb_winner) {
            (Some(a), Some(b)) => {
                write_bracket_match(db, tournament, a, Some(b), 0, "grand_final", 0).await?;
                return Ok(1);
            }
            _ => return Ok(0),
        }
    }

    // GF exists; if completed and LB-side won, spawn reset.
    if let (Some(gf), None) = (gf, reset) {
        if !gf.status.is_terminal() {
            return Ok(0);
        }
        let Some(winner) = gf.winner() else {
            return Ok(0);
        };
        // GF is laid out [WB-side, LB-side]: index 0 was the WB final winner.
        let Some(wb_side) = gf.participants.first() else {
            return Ok(0);
        };
        let lb_side = gf.participants.get(1);
        if winner.user_id == wb_side.user_id {
            return Ok(0); // WB-side won, no reset.
        }
        if let Some(lb_side) = lb_side {
            write_bracket_match(
                db,
                tournament,
                wb_side.clone(),
                Some(lb_side.clone()),
                0,
                "grand_final_reset",
                0,
            )
            .await?;
            return Ok(1);
        }
    }
    Ok(0)
}
