// Tournament matchmaking for human-vs-human flows.
//
// Players opt in to a queue per tournament. The matchmaker runs on a
// timer and pairs queued players into ranked rooms (one room per pair
// for 1v1 games). Pairings respect the participant set so a non-joiner
// can never sneak in.
//
// The pairing strategy is intentionally simple: FIFO by joined_at, ties
// broken by elo (closest pair). It's good enough for fixtures and easy
// to extend with skill-based or location-based bias later.

use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::tournament::{TournamentKind, TournamentStatus},
    services::{elo::DEFAULT_ELO, room as room_svc, tournament as tournament_svc},
};
use surrealdb::types::{Datetime, RecordId, SurrealValue};
#[cfg(test)]
use surrealdb::types::ToSql;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct MatchmakingTicket {
    pub id: Option<RecordId>,
    pub tournament_id: RecordId,
    pub user_id: RecordId,
    pub elo: f64,
    pub queued_at: Datetime,
}

/// Add the user to the ranked-match queue for this tournament. Idempotent
/// — calling twice does not create duplicates. Requires the user to be a
/// joined participant.
pub async fn enqueue(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> ApiResult<MatchmakingTicket> {
    let part = tournament_svc::get_participant(db, &tournament_id, &user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("You must join the tournament first".to_string()))?;
    let elo = part.elo.unwrap_or(DEFAULT_ELO);

    let mut existing_resp = db
        .query(
            "SELECT * FROM matchmaking_ticket
             WHERE tournament_id = $tid AND user_id = $uid LIMIT 1",
        )
        .bind(("tid", tournament_id.clone()))
        .bind(("uid", user_id.clone()))
        .await?;
    let existing: Vec<MatchmakingTicket> = existing_resp.take(0)?;
    if let Some(t) = existing.into_iter().next() {
        return Ok(t);
    }
    let ticket = MatchmakingTicket {
        id: None,
        tournament_id,
        user_id,
        elo,
        queued_at: Datetime::default(),
    };
    let created: Option<MatchmakingTicket> =
        db.create("matchmaking_ticket").content(ticket).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to enqueue".to_string()))
}

pub async fn dequeue(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> ApiResult<()> {
    db.query(
        "DELETE matchmaking_ticket
         WHERE tournament_id = $tid AND user_id = $uid",
    )
    .bind(("tid", tournament_id))
    .bind(("uid", user_id))
    .await?;
    Ok(())
}

/// Pair queued players for one tournament, creating ranked rooms for
/// each match. Returns the number of rooms created.
pub async fn tick(db: &Database, tournament_id: RecordId) -> ApiResult<u32> {
    use crate::models::tournament::Tournament;
    let tournament: Tournament = {
        let opt: Option<Tournament> = db.select(&tournament_id).await?;
        opt.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))?
    };
    if tournament.kind != TournamentKind::Human {
        return Ok(0);
    }
    if tournament.status != TournamentStatus::Running {
        return Ok(0);
    }

    let mut tickets_resp = db
        .query(
            "SELECT * FROM matchmaking_ticket
             WHERE tournament_id = $tid
             ORDER BY queued_at ASC",
        )
        .bind(("tid", tournament_id.clone()))
        .await?;
    let queue: Vec<MatchmakingTicket> = tickets_resp.take(0)?;
    let mut paired = 0u32;

    for (a, b) in pair_tickets(queue) {
        let (Some(aid), Some(bid)) = (a.id.clone(), b.id.clone()) else {
            continue;
        };
        // Atomically claim both tickets so two ticks racing on the same
        // queue can never double-pair. If we don't get exactly 2 rows
        // back, another tick beat us — skip without creating a room.
        let mut claim = db
            .query("DELETE $aid, $bid RETURN BEFORE")
            .bind(("aid", aid))
            .bind(("bid", bid))
            .await?;
        let claimed: Vec<MatchmakingTicket> = claim.take(0).unwrap_or_default();
        if claimed.len() < 2 {
            continue;
        }

        room_svc::create_ranked_room(
            db,
            tournament_id.clone(),
            vec![a.user_id.clone(), b.user_id.clone()],
        )
        .await?;
        paired += 1;
    }
    Ok(paired)
}

/// Closest-ELO pairing over a queue ordered by `queued_at`. Pure: no DB
/// calls, exposed for unit tests and reused by `tick`.
pub fn pair_tickets(mut q: Vec<MatchmakingTicket>) -> Vec<(MatchmakingTicket, MatchmakingTicket)> {
    let mut out = Vec::with_capacity(q.len() / 2);
    while q.len() >= 2 {
        let a = q.remove(0);
        let mut best = 0usize;
        let mut best_diff = f64::INFINITY;
        for (i, t) in q.iter().enumerate() {
            let d = (t.elo - a.elo).abs();
            if d < best_diff {
                best_diff = d;
                best = i;
            }
        }
        let b = q.remove(best);
        out.push((a, b));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(s: &str) -> RecordId {
        RecordId::parse_simple(&format!("user:{s}")).unwrap()
    }
    fn ticket(uid: &str, elo: f64) -> MatchmakingTicket {
        MatchmakingTicket {
            id: None,
            tournament_id: RecordId::parse_simple("tournament:t").unwrap(),
            user_id: user(uid),
            elo,
            queued_at: Datetime::default(),
        }
    }

    #[test]
    fn pair_tickets_picks_closest_elo() {
        let q = vec![
            ticket("a", 1000.0),
            ticket("b", 1500.0),
            ticket("c", 1480.0),
            ticket("d", 1010.0),
        ];
        let pairs = pair_tickets(q);
        let names: Vec<(String, String)> = pairs
            .into_iter()
            .map(|(a, b)| (a.user_id.to_sql(), b.user_id.to_sql()))
            .collect();
        assert!(names[0].0.contains(":a"));
        assert!(names[0].1.contains(":d"));
        assert!(names[1].0.contains(":b"));
        assert!(names[1].1.contains(":c"));
    }

    #[test]
    fn pair_tickets_leaves_odd_one_out() {
        let q = vec![
            ticket("a", 1000.0),
            ticket("b", 1100.0),
            ticket("c", 1200.0),
        ];
        let pairs = pair_tickets(q);
        assert_eq!(pairs.len(), 1);
    }
}
