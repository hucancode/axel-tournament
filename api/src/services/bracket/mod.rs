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

mod common;
mod double;
mod single;

use crate::{
    db::Database,
    error::AppResult,
    models::{
        matches::Match,
        tournament::{MatchGenerationType, TournamentKind},
    },
};
use axel_core::repo::tournament::TournamentRepo;
use surrealdb::types::RecordId;

pub async fn advance_brackets(db: &Database, tournament_id: RecordId) -> AppResult<u32> {
    let tournament = <Database as TournamentRepo>::get_by_id(db, &tournament_id).await?;
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
        created += single::advance_single(db, &tournament, &matches).await?;
    } else if is_double {
        created += double::advance_double(db, &tournament, &matches).await?;
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use crate::models::matches::{MatchStatus, test_helpers::finished};
    use surrealdb::types::{RecordId, ToSql};

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
