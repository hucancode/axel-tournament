// Bracket advancement.
//
// Healer / judge call `advance_brackets(tournament_id)` after match
// writes. For every completed round-N match without a child round-N+1
// match yet, generate the next-round match by pairing winners.
//
// Single elim: winners advance from `winners` bracket. When only one
// match remains and it's done, the bracket is finished — finalization
// runs the usual completion path.
//
// Double elim: winners-bracket losers drop down into the losers
// bracket. Losers-bracket losers are eliminated. Winners-bracket
// finalist plays losers-bracket finalist in the grand final, with a
// reset-bracket if losers' player wins.

use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        matches::{Match, MatchParticipant, MatchStatus},
        tournament::{MatchGenerationType, Tournament, TournamentKind},
    },
};
use surrealdb::types::{RecordId, ToSql};

pub async fn advance_brackets(db: &Database, tournament_id: RecordId) -> ApiResult<u32> {
    let tournament: Tournament = {
        let opt: Option<Tournament> = db.select(&tournament_id).await?;
        opt.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))?
    };
    if tournament.kind != TournamentKind::Bot {
        // Human bracket flows would need room creation per match; skip
        // for now.
        return Ok(0);
    }
    let is_double = tournament.match_generation_type == MatchGenerationType::DoubleElimination;
    let is_single = tournament.match_generation_type == MatchGenerationType::SingleElimination;
    if !is_single && !is_double {
        return Ok(0);
    }

    let mut resp = db
        .query("SELECT * FROM match WHERE tournament_id = $tid ORDER BY round, bracket_position")
        .bind(("tid", tournament_id.clone()))
        .await?;
    let matches: Vec<Match> = resp.take(0)?;
    let mut created = 0u32;

    if is_single {
        created += advance_single(db, &tournament, &matches).await?;
    } else if is_double {
        created += advance_double(db, &tournament, &matches).await?;
    }
    Ok(created)
}

/// Single-elim advancement: pair winners of round N's positions 2k and
/// 2k+1 into round N+1 position k. BYE matches (one participant) skip
/// to next round automatically.
async fn advance_single(
    db: &Database,
    tournament: &Tournament,
    matches: &[Match],
) -> ApiResult<u32> {
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
    if current
        .iter()
        .any(|m| !matches!(m.status, MatchStatus::Completed | MatchStatus::Failed))
    {
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
        let wa = ma.winner().ok_or_else(|| {
            ApiError::Internal(format!("match {} has no winner", ma.id.as_ref().map(|i| i.to_sql()).unwrap_or_default()))
        })?;
        let wb = mb.winner().ok_or_else(|| {
            ApiError::Internal(format!("match {} has no winner", mb.id.as_ref().map(|i| i.to_sql()).unwrap_or_default()))
        })?;
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
async fn advance_double(
    db: &Database,
    tournament: &Tournament,
    matches: &[Match],
) -> ApiResult<u32> {
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
) -> ApiResult<u32> {
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
        let pairs = lb_round_pairs(matches, lb_round, k);
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
    k: u32,
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
    let _ = k; // silenced (k currently informational)
    Some(pairs)
}

fn round_winners(
    matches: &[Match],
    bracket: &str,
    round: u32,
) -> Option<Vec<MatchParticipant>> {
    let mut rows: Vec<&Match> = matches
        .iter()
        .filter(|m| m.bracket.as_deref() == Some(bracket) && m.round == Some(round))
        .collect();
    if rows.is_empty() {
        return None;
    }
    rows.sort_by_key(|m| m.bracket_position.unwrap_or(0));
    let mut out = Vec::with_capacity(rows.len());
    for m in rows {
        let w = m.winner()?;
        out.push(w);
    }
    Some(out)
}

fn round_losers(
    matches: &[Match],
    bracket: &str,
    round: u32,
) -> Option<Vec<MatchParticipant>> {
    let mut rows: Vec<&Match> = matches
        .iter()
        .filter(|m| m.bracket.as_deref() == Some(bracket) && m.round == Some(round))
        .collect();
    if rows.is_empty() {
        return None;
    }
    rows.sort_by_key(|m| m.bracket_position.unwrap_or(0));
    let mut out = Vec::with_capacity(rows.len());
    for m in rows {
        let l = loser_of(m)?;
        out.push(l);
    }
    Some(out)
}

fn loser_of(m: &Match) -> Option<MatchParticipant> {
    if !matches!(m.status, MatchStatus::Completed | MatchStatus::Failed) {
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
) -> ApiResult<u32> {
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
    if reset.is_none() {
        let gf = gf.unwrap();
        if !matches!(gf.status, MatchStatus::Completed | MatchStatus::Failed) {
            return Ok(0);
        }
        let winner = match gf.winner() {
            Some(w) => w,
            None => return Ok(0),
        };
        // GF is laid out [WB-side, LB-side]: index 0 was the WB final winner.
        let wb_side = &gf.participants[0];
        let lb_side = &gf.participants.get(1);
        if winner.user_id == wb_side.user_id {
            return Ok(0); // WB-side won, no reset.
        }
        if let Some(lb_side) = lb_side {
            write_bracket_match(
                db,
                tournament,
                wb_side.clone(),
                Some((*lb_side).clone()),
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

async fn write_bracket_match(
    db: &Database,
    tournament: &Tournament,
    p1: MatchParticipant,
    p2: Option<MatchParticipant>,
    round: u32,
    bracket: &str,
    position: u32,
) -> ApiResult<()> {
    let tournament_id = tournament
        .id
        .clone()
        .ok_or_else(|| ApiError::Internal("Tournament missing id".to_string()))?;
    let mut participants = vec![p1];
    if let Some(p) = p2 {
        participants.push(p);
    }
    let m = Match {
        tournament_id: Some(tournament_id),
        game_id: tournament.game_id.clone(),
        participants,
        round: Some(round),
        bracket: Some(bracket.to_string()),
        bracket_position: Some(position),
        ..Default::default()
    };
    let _: Option<Match> = db.create("match").content(m).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::matches::test_helpers::finished;
    use crate::services::tournament::single_elim_round_zero;

    #[test]
    fn round_zero_two_players_pairs_them() {
        let pairs = single_elim_round_zero(2);
        assert_eq!(pairs, vec![(0, Some(1))]);
    }

    #[test]
    fn round_zero_eight_players_uses_standard_seeding() {
        let pairs = single_elim_round_zero(8);
        // (1,8), (4,5), (2,7), (3,6) by 1-vs-N — zero-indexed:
        assert_eq!(
            pairs,
            vec![(0, Some(7)), (3, Some(4)), (1, Some(6)), (2, Some(5))]
        );
    }

    #[test]
    fn round_zero_three_players_gives_top_seed_a_bye() {
        let pairs = single_elim_round_zero(3);
        // Bracket 4. Seed 0 vs (3=BYE) -> top seed bye; (1,2) plays.
        assert_eq!(pairs[0], (0, None));
        assert_eq!(pairs[1], (1, Some(2)));
    }

    #[test]
    fn round_zero_five_players_two_get_byes() {
        let pairs = single_elim_round_zero(5);
        let byes = pairs.iter().filter(|(_, b)| b.is_none()).count();
        assert_eq!(byes, 3);
        let played = pairs.iter().filter(|(_, b)| b.is_some()).count();
        assert_eq!(played, 1);
    }

    #[test]
    fn winner_of_completed_picks_higher_score() {
        let m = finished("alice", 3.0, "bob", 1.0);
        assert_eq!(m.winner().unwrap().user_id.to_sql(), "user:alice");
    }

    #[test]
    fn winner_of_failed_match_picks_non_faulted() {
        let mut m = finished("alice", 0.0, "bob", 0.0);
        m.status = MatchStatus::Failed;
        m.faulted_user_ids = vec![RecordId::parse_simple("user:alice").unwrap()];
        assert_eq!(m.winner().unwrap().user_id.to_sql(), "user:bob");
    }

    #[test]
    fn winner_of_failed_with_both_at_fault_is_none() {
        let mut m = finished("alice", 0.0, "bob", 0.0);
        m.status = MatchStatus::Failed;
        m.faulted_user_ids = vec![
            RecordId::parse_simple("user:alice").unwrap(),
            RecordId::parse_simple("user:bob").unwrap(),
        ];
        assert!(m.winner().is_none());
    }
}
