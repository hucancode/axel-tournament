use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        matches::{Match, MatchParticipant, MatchStatus},
        tournament::{MatchGenerationType, Tournament, TournamentParticipant},
    },
};
use surrealdb::types::Datetime;

/// Dispatch to the right match-generation strategy. Returns the count of
/// matches created.
pub async fn generate_matches(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    match tournament.match_generation_type {
        MatchGenerationType::AllVsAll => {
            generate_pairwise(db, tournament, participants, /*include_self_and_dupes=*/ true)
                .await
        }
        MatchGenerationType::RoundRobin => {
            generate_pairwise(db, tournament, participants, false).await
        }
        MatchGenerationType::SingleElimination
        | MatchGenerationType::DoubleElimination => {
            generate_elimination_round_zero(db, tournament, participants).await
        }
    }
}

/// All-vs-all (NxN, includes self-play and reverse) when
/// `include_self_and_dupes` is true; otherwise round-robin
/// (`N*(N-1)/2`, unique unordered pairs).
async fn generate_pairwise(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
    include_self_and_dupes: bool,
) -> ApiResult<usize> {
    let mut created = 0;
    if include_self_and_dupes {
        for p1 in participants {
            for p2 in participants {
                create_match_for_participants(db, tournament, p1, p2).await?;
                created += 1;
            }
        }
    } else {
        for i in 0..participants.len() {
            for j in (i + 1)..participants.len() {
                create_match_for_participants(db, tournament, &participants[i], &participants[j])
                    .await?;
                created += 1;
            }
        }
    }
    Ok(created)
}

/// Single- and double-elim share round-0 generation. Healer / judge
/// generates the next rounds as round N completes (in `services::bracket`).
async fn generate_elimination_round_zero(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    let pairs = single_elim_round_zero(participants.len());
    let mut created = 0;
    for (pos, (a, b)) in pairs.iter().enumerate() {
        let p1 = &participants[*a];
        let p2_opt = b.map(|i| &participants[i]);
        create_bracket_match(db, tournament, p1, p2_opt, 0, "winners", pos as u32).await?;
        created += 1;
    }
    Ok(created)
}

async fn create_match_for_participants(
    db: &Database,
    tournament: &Tournament,
    p1: &TournamentParticipant,
    p2: &TournamentParticipant,
) -> ApiResult<()> {
    let submission_id_1 = p1
        .submission_id
        .clone()
        .ok_or_else(|| ApiError::Internal("Participant missing submission".to_string()))?;
    let submission_id_2 = p2
        .submission_id
        .clone()
        .ok_or_else(|| ApiError::Internal("Participant missing submission".to_string()))?;

    let tournament_id = tournament
        .id
        .clone()
        .ok_or_else(|| ApiError::Internal("Tournament missing id".to_string()))?;
    let match_data = Match {
        tournament_id: Some(tournament_id),
        game_id: tournament.game_id.clone(),
        participants: vec![
            MatchParticipant {
                user_id: p1.user_id.clone(),
                submission_id: Some(submission_id_1),
                score: None,
            },
            MatchParticipant {
                user_id: p2.user_id.clone(),
                submission_id: Some(submission_id_2),
                score: None,
            },
        ],
        ..Default::default()
    };
    let _: Option<Match> = db.create("match").content(match_data).await?;
    Ok(())
}

/// Standard 1-vs-N seeding. For 8 players: (0,7), (3,4), (2,5), (1,6).
/// For non-power-of-2, top seeds receive byes (b = None).
pub fn single_elim_round_zero(n: usize) -> Vec<(usize, Option<usize>)> {
    if n < 2 {
        return Vec::new();
    }
    let bracket_size = n.next_power_of_two();
    let order = seed_order(bracket_size);
    let mut pairs = Vec::with_capacity(bracket_size / 2);
    for chunk in order.chunks(2) {
        let a = chunk[0];
        let b = chunk[1];
        let a_real = if a < n { Some(a) } else { None };
        let b_real = if b < n { Some(b) } else { None };
        match (a_real, b_real) {
            (Some(ai), Some(bi)) => pairs.push((ai, Some(bi))),
            (Some(ai), None) => pairs.push((ai, None)),
            (None, Some(bi)) => pairs.push((bi, None)),
            (None, None) => {} // both byes, drop
        }
    }
    pairs
}

/// Iterative seed order. For 4: [0,3,1,2]. For 8: [0,7,3,4,1,6,2,5].
pub fn seed_order(size: usize) -> Vec<usize> {
    let mut v = vec![0usize, 1];
    let mut s = 2usize;
    while s < size {
        let mut next = Vec::with_capacity(s * 2);
        for &x in &v {
            next.push(x);
            next.push(s * 2 - 1 - x);
        }
        v = next;
        s *= 2;
    }
    v
}

async fn create_bracket_match(
    db: &Database,
    tournament: &Tournament,
    p1: &TournamentParticipant,
    p2: Option<&TournamentParticipant>,
    round: u32,
    bracket: &str,
    position: u32,
) -> ApiResult<()> {
    let tournament_id = tournament
        .id
        .clone()
        .ok_or_else(|| ApiError::Internal("Tournament missing id".to_string()))?;

    let mut participants = vec![MatchParticipant {
        user_id: p1.user_id.clone(),
        submission_id: p1.submission_id.clone(),
        score: None,
    }];
    if let Some(p2) = p2 {
        participants.push(MatchParticipant {
            user_id: p2.user_id.clone(),
            submission_id: p2.submission_id.clone(),
            score: None,
        });
    }

    // BYE: only one participant. Mark Completed immediately so bracket
    // advancement picks the winner up on the next healer tick.
    let (status, score, completed_at) = if p2.is_none() {
        (MatchStatus::Completed, Some(1.0_f64), Some(Datetime::default()))
    } else {
        (MatchStatus::Pending, None, None)
    };
    if let Some(s) = score {
        participants[0].score = Some(s);
    }

    let match_data = Match {
        tournament_id: Some(tournament_id),
        game_id: tournament.game_id.clone(),
        status,
        participants,
        round: Some(round),
        bracket: Some(bracket.to_string()),
        bracket_position: Some(position),
        completed_at,
        ..Default::default()
    };
    let _: Option<Match> = db.create("match").content(match_data).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_zero_two_players_pairs_them() {
        let pairs = single_elim_round_zero(2);
        assert_eq!(pairs, vec![(0, Some(1))]);
    }

    #[test]
    fn round_zero_eight_players_uses_standard_seeding() {
        let pairs = single_elim_round_zero(8);
        assert_eq!(
            pairs,
            vec![(0, Some(7)), (3, Some(4)), (1, Some(6)), (2, Some(5))]
        );
    }

    #[test]
    fn round_zero_three_players_gives_top_seed_a_bye() {
        let pairs = single_elim_round_zero(3);
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
}
