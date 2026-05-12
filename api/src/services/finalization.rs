// Tournament finalization.
//
// After every match write, judges (or the healer cron) call
// `finalize_if_done`. It:
//   * Aggregates terminal-match scores into `tournament_participant.score`
//     and `wins`/`losses`/`draws` for the active submission.
//   * If no matches remain in pending/queued/running state, flips the
//     tournament to `Completed` and assigns ranks by score.
//
// Idempotent: running it twice produces the same result, since scores
// are recomputed from match rows each call.

use crate::{
    db::Database,
    error::AppResult,
    models::{
        matches::{Match, MatchStatus},
        tournament::{Tournament, TournamentStatus},
    },
};
use std::collections::HashMap;
use surrealdb::types::{RecordId, ToSql};

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantTotals {
    pub user_id: String,
    pub score: f64,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

/// Pure aggregation: turn a list of terminal matches into per-user totals.
/// Exposed so unit tests can verify the win/loss/draw rule independently
/// of the database layer.
pub fn aggregate_totals(matches: &[Match]) -> HashMap<String, ParticipantTotals> {
    let mut totals: HashMap<String, ParticipantTotals> = HashMap::new();
    fn entry_for<'a>(
        totals: &'a mut HashMap<String, ParticipantTotals>,
        uid: &str,
    ) -> &'a mut ParticipantTotals {
        totals
            .entry(uid.to_string())
            .or_insert_with(|| ParticipantTotals {
                user_id: uid.to_string(),
                score: 0.0,
                wins: 0,
                losses: 0,
                draws: 0,
            })
    }

    for m in matches {
        if m.status != MatchStatus::Completed && m.status != MatchStatus::Failed {
            continue;
        }
        let faulted: std::collections::HashSet<String> =
            m.faulted_user_ids.iter().map(|u| u.to_sql()).collect();

        // Faulted-only path: at least one participant is at fault. Each
        // faulted user takes a loss, every other participant takes a
        // win (the bot stayed within the rules). This applies to runtime
        // errors, illegal moves, compile fails, disconnects.
        if !faulted.is_empty() {
            for p in &m.participants {
                let uid = p.user_id.to_sql();
                let entry = entry_for(&mut totals, &uid);
                if let Some(s) = p.score {
                    entry.score += s;
                }
                if faulted.contains(&uid) {
                    entry.losses += 1;
                } else {
                    entry.wins += 1;
                }
            }
            continue;
        }

        // Whole-match Failed with nobody at fault: legacy fallback —
        // count every side as a loss so a broken match never looks
        // like a free win.
        if m.status == MatchStatus::Failed {
            for p in &m.participants {
                let uid = p.user_id.to_sql();
                entry_for(&mut totals, &uid).losses += 1;
            }
            continue;
        }

        // Normal completion: w/l/d by score comparison; total_score sums.
        let scores: Vec<f64> = m
            .participants
            .iter()
            .map(|p| p.score.unwrap_or(0.0))
            .collect();
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let leaders = scores.iter().filter(|s| **s == max).count();

        for (i, p) in m.participants.iter().enumerate() {
            let uid = p.user_id.to_sql();
            let entry = entry_for(&mut totals, &uid);
            entry.score += scores[i];
            if scores[i] == max && leaders == 1 {
                entry.wins += 1;
            } else if scores[i] == max && leaders > 1 {
                entry.draws += 1;
            } else {
                entry.losses += 1;
            }
        }
    }
    totals
}

/// Read every terminal match for the tournament and write aggregated
/// totals onto each participant. Then mark the tournament `Completed`
/// (with ranks) iff no non-terminal match remains.
pub async fn finalize_if_done(
    db: &Database,
    tournament_id: RecordId,
) -> AppResult<Tournament> {
    let mut matches_resp = db
        .query("SELECT * FROM match WHERE tournament_id = $tid")
        .bind(("tid", tournament_id.clone()))
        .await?;
    let matches: Vec<Match> = matches_resp.take(0)?;

    let totals = aggregate_totals(&matches);

    // Push totals onto each participant.
    let participants =
        crate::services::tournament::get_tournament_participants(db, tournament_id.clone()).await?;

    for p in &participants {
        let uid = p.user_id.to_sql();
        let t = totals.get(&uid).cloned().unwrap_or(ParticipantTotals {
            user_id: uid,
            score: 0.0,
            wins: 0,
            losses: 0,
            draws: 0,
        });
        if let Some(pid) = p.id.clone() {
            db.query(
                "UPDATE $pid SET score = $s, wins = $w, losses = $l, draws = $d",
            )
            .bind(("pid", pid))
            .bind(("s", t.score))
            .bind(("w", t.wins as i64))
            .bind(("l", t.losses as i64))
            .bind(("d", t.draws as i64))
            .await?;
        }
    }

    // Any non-terminal match left? If so, tournament still running.
    let remaining = matches
        .iter()
        .filter(|m| {
            matches!(
                m.status,
                MatchStatus::Pending | MatchStatus::Queued | MatchStatus::Running
            )
        })
        .count();

    // Continuous tournaments never finalize on "no matches remaining" —
    // players keep enqueueing until end_time. Only end_time triggers
    // finalization for that mode.
    let tournament_now =
        <Database as axel_core::repo::tournament::TournamentRepo>::get_by_id(db, &tournament_id)
            .await?;
    if tournament_now.match_generation_type
        == crate::models::tournament::MatchGenerationType::Continuous
    {
        let past_end = tournament_now
            .end_time
            .as_ref()
            .map(|et| {
                let now = surrealdb::types::Datetime::default();
                et.to_string() <= now.to_string()
            })
            .unwrap_or(false);
        if !past_end {
            return Ok(tournament_now);
        }
    } else if remaining > 0 {
        return Ok(tournament_now);
    }

    // No remaining matches — assign ranks by score (desc, ties share rank).
    let mut sorted = participants;
    sorted.sort_by(|a, b| {
        let sa = totals.get(&a.user_id.to_sql()).map(|t| t.score).unwrap_or(0.0);
        let sb = totals.get(&b.user_id.to_sql()).map(|t| t.score).unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut last_score: Option<f64> = None;
    let mut rank = 0u32;
    for (i, p) in sorted.iter().enumerate() {
        let s = totals.get(&p.user_id.to_sql()).map(|t| t.score).unwrap_or(0.0);
        if Some(s) != last_score {
            rank = (i as u32) + 1;
            last_score = Some(s);
        }
        if let Some(pid) = p.id.clone() {
            db.query("UPDATE $pid SET rank = $r")
                .bind(("pid", pid))
                .bind(("r", rank as i64))
                .await?;
        }
    }

    let mut updated = db
        .query(
            "UPDATE $tid SET status = 'completed', updated_at = time::now(),
                              end_time = time::now()
             WHERE status IN ['running', 'generating']
             RETURN AFTER",
        )
        .bind(("tid", tournament_id.clone()))
        .await?;
    let rows: Vec<Tournament> = updated.take(0)?;
    if let Some(t) = rows.into_iter().next() {
        return Ok(t);
    }
    <Database as axel_core::repo::tournament::TournamentRepo>::get_by_id(db, &tournament_id).await
}

/// Return the status the tournament should hold given its match set.
/// Used for tests + the healer.
pub fn implied_status(matches: &[Match]) -> Option<TournamentStatus> {
    let total = matches.len();
    if total == 0 {
        return None;
    }
    let remaining = matches
        .iter()
        .filter(|m| {
            matches!(
                m.status,
                MatchStatus::Pending | MatchStatus::Queued | MatchStatus::Running
            )
        })
        .count();
    if remaining == 0 {
        Some(TournamentStatus::Completed)
    } else {
        Some(TournamentStatus::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::matches::test_helpers::{failed, faulted, finished, pending};

    #[test]
    fn aggregate_win_loss_draw_basic() {
        let ms = vec![
            finished("a", 3.0, "b", 1.0), // a wins
            finished("a", 1.0, "b", 1.0), // draw
            finished("a", 0.0, "b", 5.0), // b wins
        ];
        let t = aggregate_totals(&ms);
        let a = &t["user:a"];
        let b = &t["user:b"];
        assert_eq!((a.wins, a.losses, a.draws), (1, 1, 1));
        assert_eq!((b.wins, b.losses, b.draws), (1, 1, 1));
        assert_eq!(a.score, 4.0);
        assert_eq!(b.score, 7.0);
    }

    #[test]
    fn aggregate_failed_match_counts_loss_for_all() {
        let ms = vec![failed("a", "b")];
        let t = aggregate_totals(&ms);
        assert_eq!(t["user:a"].losses, 1);
        assert_eq!(t["user:b"].losses, 1);
        assert_eq!(t["user:a"].score, 0.0);
    }

    #[test]
    fn aggregate_runtime_error_punishes_only_faulted_bot() {
        let ms = vec![faulted("a", "b", "a")]; // a crashed
        let t = aggregate_totals(&ms);
        assert_eq!(t["user:a"].losses, 1);
        assert_eq!(t["user:a"].wins, 0);
        assert_eq!(t["user:b"].wins, 1);
        assert_eq!(t["user:b"].losses, 0);
    }

    #[test]
    fn aggregate_illegal_move_same_as_runtime_error() {
        // Two-bot match where b makes an illegal move. b loses, a wins.
        let ms = vec![faulted("a", "b", "b")];
        let t = aggregate_totals(&ms);
        assert_eq!(t["user:b"].losses, 1);
        assert_eq!(t["user:a"].wins, 1);
    }

    #[test]
    fn aggregate_skips_pending() {
        let ms = vec![pending("a", "b"), finished("a", 1.0, "b", 0.0)];
        let t = aggregate_totals(&ms);
        assert_eq!(t["user:a"].wins, 1);
    }

    #[test]
    fn implied_status_running_when_pending() {
        let ms = vec![pending("a", "b"), finished("a", 1.0, "b", 0.0)];
        assert_eq!(implied_status(&ms), Some(TournamentStatus::Running));
    }

    #[test]
    fn implied_status_completed_when_all_terminal() {
        let ms = vec![
            finished("a", 1.0, "b", 0.0),
            failed("a", "b"),
        ];
        assert_eq!(implied_status(&ms), Some(TournamentStatus::Completed));
    }
}
