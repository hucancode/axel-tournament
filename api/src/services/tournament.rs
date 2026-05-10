use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        tournament::{
            MatchGenerationType, Tournament, TournamentKind, TournamentParticipant, TournamentStatus,
        },
        matches::{Match, MatchParticipant, MatchStatus},
        game::find_game_by_id,
    },
};
use chrono::{DateTime, Utc};
use surrealdb::types::{Datetime, RecordId};

pub async fn create_tournament(
    db: &Database,
    game_id: String,
    name: String,
    description: String,
    min_players: u32,
    max_players: u32,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    match_generation_type: Option<MatchGenerationType>,
    kind: Option<TournamentKind>,
) -> ApiResult<Tournament> {
    // Verify game exists in hardcoded registry
    find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    let tournament = Tournament {
        id: None,
        game_id,
        name,
        description,
        status: TournamentStatus::Registration,
        min_players,
        max_players,
        start_time: start_time.map(|dt| dt.into()),
        end_time: end_time.map(|dt| dt.into()),
        match_generation_type: match_generation_type.unwrap_or_default(),
        kind: kind.unwrap_or_default(),
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Tournament> = db.create("tournament").content(tournament).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create tournament".to_string()))
}

pub async fn get_tournament(db: &Database, tournament_id: RecordId) -> ApiResult<Tournament> {
    let tournament: Option<Tournament> = db.select(&tournament_id).await?;
    tournament.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

pub async fn list_tournaments(
    db: &Database,
    status: Option<TournamentStatus>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> ApiResult<Vec<Tournament>> {
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);
    let mut result = if let Some(s) = status {
        let status_str = serde_json::to_string(&s)
            .unwrap()
            .trim_matches('"')
            .to_string();
        db.query(
            "SELECT * FROM tournament
             WHERE status = $status
             ORDER BY created_at DESC
             LIMIT $limit START $offset",
        )
            .bind(("status", status_str))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?
    } else {
        db.query(
            "SELECT * FROM tournament
             ORDER BY created_at DESC
             LIMIT $limit START $offset",
        )
        .bind(("limit", limit))
        .bind(("offset", offset))
            .await?
    };
    let tournaments: Vec<Tournament> = result.take(0)?;
    Ok(tournaments)
}

pub async fn update_tournament(
    db: &Database,
    tournament_id: RecordId,
    name: Option<String>,
    description: Option<String>,
    status: Option<TournamentStatus>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> ApiResult<Tournament> {
    let mut tournament = get_tournament(db, tournament_id.clone()).await?;
    if let Some(n) = name {
        tournament.name = n;
    }
    if let Some(d) = description {
        tournament.description = d;
    }
    if let Some(s) = status {
        tournament.status = s;
    }
    if let Some(st) = start_time {
        tournament.start_time = Some(st.into());
    }
    if let Some(et) = end_time {
        tournament.end_time = Some(et.into());
    }
    tournament.updated_at = Datetime::default();
    let updated: Option<Tournament> = db.update(&tournament_id).content(tournament).await?;
    updated.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

pub async fn join_tournament(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> ApiResult<TournamentParticipant> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    // Check if tournament is accepting registrations
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Tournament is not accepting registrations".to_string(),
        ));
    }

    // Get current participants count
    let participants = get_tournament_participants(db, tournament_id.clone()).await?;

    // Check if tournament is full
    if participants.len() as u32 >= tournament.max_players {
        return Err(ApiError::BadRequest("Tournament is full".to_string()));
    }

    // Check if user already joined
    if participants.iter().any(|p| p.user_id == user_id) {
        return Err(ApiError::Conflict(
            "User already joined this tournament".to_string(),
        ));
    }

    let participant = TournamentParticipant {
        id: None,
        tournament_id: tournament_id.clone(),
        user_id: user_id.clone(),
        submission_id: None,
        score: 0.0,
        wins: 0,
        losses: 0,
        draws: 0,
        elo: None,
        rank: None,
        joined_at: Datetime::default(),
    };
    let created: Option<TournamentParticipant> = db
        .create("tournament_participant")
        .content(participant)
        .await?;
    created.ok_or_else(|| ApiError::Internal("Failed to join tournament".to_string()))
}

pub async fn get_tournament_participants(
    db: &Database,
    tournament_id: RecordId,
) -> ApiResult<Vec<TournamentParticipant>> {
    let mut result = db
        .query("SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id ORDER BY score DESC")
        .bind(("tournament_id", tournament_id))
        .await?;
    let participants: Vec<TournamentParticipant> = result.take(0)?;
    Ok(participants)
}

/// Single participant lookup by `(tournament, user)`. None if absent.
pub async fn get_participant(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
) -> ApiResult<Option<TournamentParticipant>> {
    let mut resp = db
        .query(
            "SELECT * FROM tournament_participant
             WHERE tournament_id = $tid AND user_id = $uid LIMIT 1",
        )
        .bind(("tid", tournament_id.clone()))
        .bind(("uid", user_id.clone()))
        .await?;
    let rows: Vec<TournamentParticipant> = resp.take(0)?;
    Ok(rows.into_iter().next())
}

pub async fn leave_tournament(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> ApiResult<()> {
    // Check tournament status - cannot leave if tournament has started
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Cannot leave tournament after registration has closed".to_string(),
        ));
    }
    let participant = get_participant(db, &tournament_id, &user_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound("You are not a participant in this tournament".to_string())
        })?;

    // Delete by specific participant id
    if let Some(pid) = participant.id.clone() {
        let _: Option<TournamentParticipant> = db.delete(&pid).await?;
    }
    Ok(())
}

/// Start a tournament and generate matches based on the configured match generation type
pub async fn start_tournament(db: &Database, tournament_id: RecordId) -> ApiResult<Tournament> {
    let tournament_id_thing = tournament_id.clone();
    let tournament = get_tournament(db, tournament_id_thing.clone()).await?;

    // Check if tournament is in registration state
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Tournament must be in registration state to start".to_string(),
        ));
    }

    // Get all participants with their submissions
    let participants = get_tournament_participants(db, tournament_id_thing.clone()).await?;

    // Check if minimum players requirement is met
    if tournament.min_players > participants.len() as u32 {
        return Err(ApiError::BadRequest(format!(
            "Not enough players. Need at least {} players, currently have {}",
            tournament.min_players, participants.len()
        )));
    }

    // Filter participants whose selected submission compiled cleanly.
    // Failed-compile bots never enter matches — that's the contract.
    let mut participants_with_submissions: Vec<TournamentParticipant> = Vec::new();
    for p in participants.into_iter() {
        let Some(sid) = p.submission_id.clone() else {
            continue;
        };
        let sub: Option<crate::models::Submission> = db.select(&sid).await?;
        let Some(sub) = sub else { continue };
        if sub.status == crate::models::SubmissionStatus::Accepted {
            participants_with_submissions.push(p);
        }
    }

    if participants_with_submissions.is_empty() {
        return Err(ApiError::BadRequest(
            "No participants with a compiled submission. Wait for compilation to finish or upload a fixed bot.".to_string(),
        ));
    }
    if (participants_with_submissions.len() as u32) < tournament.min_players {
        return Err(ApiError::BadRequest(format!(
            "Not enough compiled submissions. Need {}, have {}.",
            tournament.min_players,
            participants_with_submissions.len()
        )));
    }

    // Claim tournament start to prevent duplicate match generation
    let mut claimed = db
        .query(
            "UPDATE $tournament_id
             SET status = 'generating', updated_at = time::now()
             WHERE status = 'registration'
             RETURN AFTER",
        )
        .bind(("tournament_id", tournament_id_thing.clone()))
        .await?;
    let claimed_rows: Vec<Tournament> = claimed.take(0)?;
    if claimed_rows.is_empty() {
        return Err(ApiError::BadRequest(
            "Tournament has already been started".to_string(),
        ));
    }

    // Generate matches based on the match generation type
    let matches_created = match tournament.match_generation_type {
        MatchGenerationType::AllVsAll => {
            generate_all_vs_all_matches(db, &tournament, &participants_with_submissions).await
        }
        MatchGenerationType::RoundRobin => {
            generate_round_robin_matches(db, &tournament, &participants_with_submissions).await
        }
        MatchGenerationType::SingleElimination => {
            generate_single_elimination_matches(db, &tournament, &participants_with_submissions)
                .await
        }
        MatchGenerationType::DoubleElimination => {
            generate_double_elimination_matches(db, &tournament, &participants_with_submissions)
                .await
        }
    };
    let matches_created = match matches_created {
        Ok(count) => count,
        Err(err) => {
            let _ = db
                .query("DELETE match WHERE tournament_id = $tournament_id")
                .bind(("tournament_id", tournament_id_thing.clone()))
                .await;
            let _ = db
                .query(
                    "UPDATE $tournament_id
                     SET status = 'registration', updated_at = time::now()",
                )
                .bind(("tournament_id", tournament_id_thing.clone()))
                .await;
            return Err(err);
        }
    };

    if matches_created == 0 {
        // Roll back claim if no matches were generated
        let _ = db
            .query("DELETE match WHERE tournament_id = $tournament_id")
            .bind(("tournament_id", tournament_id_thing.clone()))
            .await;
        let _ = db
            .query(
                "UPDATE $tournament_id
                 SET status = 'registration', updated_at = time::now()",
            )
            .bind(("tournament_id", tournament_id_thing.clone()))
            .await;
        return Err(ApiError::Internal(
            "No matches were generated for this tournament".to_string(),
        ));
    }

    let mut updated = db
        .query(
            "UPDATE $tournament_id
             SET status = 'running', updated_at = time::now()
             WHERE status = 'generating'
             RETURN AFTER",
        )
        .bind(("tournament_id", tournament_id_thing.clone()))
        .await?;
    let updated_rows: Vec<Tournament> = updated.take(0)?;
    if let Some(tournament) = updated_rows.into_iter().next() {
        return Ok(tournament);
    }
    let updated_tournament = get_tournament(db, tournament_id_thing.clone()).await?;
    Ok(updated_tournament)
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

/// Generate all vs all matches (each player plays against every player including themselves)
/// For N players: N × N matches
async fn generate_all_vs_all_matches(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    let mut matches_created = 0;

    for p1 in participants {
        for p2 in participants {
            create_match_for_participants(db, tournament, p1, p2).await?;
            matches_created += 1;
        }
    }

    Ok(matches_created)
}

/// Generate round robin matches (each player plays against every other player once, excluding themselves)
/// For N players: N × (N-1) / 2 matches (only unique pairings, no duplicates)
async fn generate_round_robin_matches(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    let mut matches_created = 0;

    // Only create matches for i < j to avoid duplicates
    for i in 0..participants.len() {
        for j in (i + 1)..participants.len() {
            let p1 = &participants[i];
            let p2 = &participants[j];

            create_match_for_participants(db, tournament, p1, p2).await?;
            matches_created += 1;
        }
    }

    Ok(matches_created)
}

/// Single elimination: pair players for round 0; healer / judge will
/// generate round N+1 once round N is complete. Non-power-of-2 fields
/// get byes — top-seeded players are paired against placeholder
/// "BYE" slots (encoded as a one-participant match that auto-resolves
/// in their favour).
///
/// Returns the count of round-0 matches created.
async fn generate_single_elimination_matches(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    let pairs = single_elim_round_zero(participants.len());
    let mut created = 0;
    for (pos, (a, b)) in pairs.iter().enumerate() {
        let p1 = &participants[*a];
        let p2_opt = b.map(|i| &participants[i]);
        create_bracket_match(
            db,
            tournament,
            p1,
            p2_opt,
            0,
            "winners",
            pos as u32,
        )
        .await?;
        created += 1;
    }
    Ok(created)
}

/// Double elimination: round 0 of the winners bracket only. The losers
/// bracket is materialised as players drop down, by `advance_brackets`
/// in `services::bracket`.
async fn generate_double_elimination_matches(
    db: &Database,
    tournament: &Tournament,
    participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    // Round 0 is identical to single-elim; difference is downstream
    // routing of losers.
    let pairs = single_elim_round_zero(participants.len());
    let mut created = 0;
    for (pos, (a, b)) in pairs.iter().enumerate() {
        let p1 = &participants[*a];
        let p2_opt = b.map(|i| &participants[i]);
        create_bracket_match(
            db,
            tournament,
            p1,
            p2_opt,
            0,
            "winners",
            pos as u32,
        )
        .await?;
        created += 1;
    }
    Ok(created)
}

/// Standard 1-vs-N seeding. For 8 players: (0,7), (3,4), (2,5), (1,6).
/// For non-power-of-2, top seeds receive byes (b = None).
pub fn single_elim_round_zero(n: usize) -> Vec<(usize, Option<usize>)> {
    if n < 2 {
        return Vec::new();
    }
    // Bracket size = next power of two.
    let bracket_size = n.next_power_of_two();
    // Standard seeding order for that size.
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
fn seed_order(size: usize) -> Vec<usize> {
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

    // BYE: only one participant. Mark the match Completed immediately,
    // award the score, so bracket advancement picks the winner up on
    // the next healer tick.
    let (status, score, completed_at) = if p2.is_none() {
        (
            MatchStatus::Completed,
            Some(1.0_f64),
            Some(Datetime::default()),
        )
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
