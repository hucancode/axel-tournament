use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{
        Match, MatchGenerationType, MatchParticipant, MatchStatus, Tournament,
        TournamentParticipant, TournamentStatus,
    },
};
use chrono::{DateTime, Utc};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_tournament(
    db: &Database,
    game_id: Thing,
    name: String,
    description: String,
    min_players: u32,
    max_players: u32,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    match_generation_type: Option<MatchGenerationType>,
) -> ApiResult<Tournament> {
    let tournament = Tournament {
        id: None,
        game_id,
        name,
        description,
        status: TournamentStatus::Registration,
        min_players,
        max_players,
        current_players: 0,
        start_time: start_time.map(|dt| dt.into()),
        end_time: end_time.map(|dt| dt.into()),
        match_generation_type: match_generation_type.unwrap_or_default(),
        matches_generated: false,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Tournament> = db.create("tournament").content(tournament).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create tournament".to_string()))
}

pub async fn get_tournament(db: &Database, tournament_id: Thing) -> ApiResult<Tournament> {
    let key = (tournament_id.tb.as_str(), tournament_id.id.to_string());
    let tournament: Option<Tournament> = db.select(key).await?;
    tournament.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

pub async fn list_tournaments(
    db: &Database,
    status: Option<TournamentStatus>,
) -> ApiResult<Vec<Tournament>> {
    let mut result = if let Some(s) = status {
        let status_str = serde_json::to_string(&s)
            .unwrap()
            .trim_matches('"')
            .to_string();
        db.query("SELECT * FROM tournament WHERE status = $status ORDER BY created_at DESC")
            .bind(("status", status_str))
            .await?
    } else {
        db.query("SELECT * FROM tournament ORDER BY created_at DESC")
            .await?
    };
    let tournaments: Vec<Tournament> = result.take(0)?;
    Ok(tournaments)
}

pub async fn update_tournament(
    db: &Database,
    tournament_id: Thing,
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
    let key = (tournament_id.tb.as_str(), tournament_id.id.to_string());
    let updated: Option<Tournament> = db.update(key).content(tournament).await?;
    updated.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

pub async fn join_tournament(
    db: &Database,
    tournament_id: Thing,
    user_id: Thing,
) -> ApiResult<TournamentParticipant> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    // Check if tournament is accepting registrations
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Tournament is not accepting registrations".to_string(),
        ));
    }
    // Check if tournament is full
    if tournament.current_players >= tournament.max_players {
        return Err(ApiError::BadRequest("Tournament is full".to_string()));
    }
    // Check if user already joined
    let mut existing = db
        .query("SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id AND user_id = $user_id")
        .bind(("tournament_id", tournament_id.clone()))
        .bind(("user_id", user_id.clone()))
        .await?;
    let existing_participants: Vec<TournamentParticipant> = existing.take(0)?;
    if !existing_participants.is_empty() {
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
        rank: None,
        joined_at: Datetime::default(),
    };
    let created: Option<TournamentParticipant> = db
        .create("tournament_participant")
        .content(participant)
        .await?;
    // Increment current_players
    db.query("UPDATE $tournament_id SET current_players += 1")
        .bind(("tournament_id", tournament_id))
        .await?;
    created.ok_or_else(|| ApiError::Internal("Failed to join tournament".to_string()))
}

pub async fn get_tournament_participants(
    db: &Database,
    tournament_id: Thing,
) -> ApiResult<Vec<TournamentParticipant>> {
    let mut result = db
        .query("SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id ORDER BY score DESC")
        .bind(("tournament_id", tournament_id))
        .await?;
    let participants: Vec<TournamentParticipant> = result.take(0)?;
    Ok(participants)
}

pub async fn leave_tournament(
    db: &Database,
    tournament_id: Thing,
    user_id: Thing,
) -> ApiResult<()> {
    // Check tournament status - cannot leave if tournament has started
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Cannot leave tournament after registration has closed".to_string(),
        ));
    }
    // Find participant first to avoid matching issues and validate ownership
    let mut existing = db
        .query("SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id AND user_id = $user_id")
        .bind(("tournament_id", tournament_id.clone()))
        .bind(("user_id", user_id.clone()))
        .await?;
    let participants: Vec<TournamentParticipant> = existing.take(0)?;
    let participant = participants.into_iter().next().ok_or_else(|| {
        ApiError::NotFound("You are not a participant in this tournament".to_string())
    })?;

    // Delete by specific participant id
    if let Some(pid) = participant.id.clone() {
        let delete_key = (pid.tb.as_str(), pid.id.to_string());
        let _: Option<TournamentParticipant> = db.delete(delete_key).await?;
    }
    // Decrement current_players
    db.query("UPDATE $tournament_id SET current_players -= 1")
        .bind(("tournament_id", tournament_id))
        .await?;
    Ok(())
}

/// Start a tournament and generate matches based on the configured match generation type
pub async fn start_tournament(db: &Database, tournament_id: Thing) -> ApiResult<Tournament> {
    let tournament_id_thing = tournament_id.clone();
    let mut tournament = get_tournament(db, tournament_id_thing.clone()).await?;

    // Check if tournament is in registration state
    if tournament.status != TournamentStatus::Registration {
        return Err(ApiError::BadRequest(
            "Tournament must be in registration state to start".to_string(),
        ));
    }

    // Check if matches have already been generated
    if tournament.matches_generated {
        return Err(ApiError::BadRequest(
            "Matches have already been generated for this tournament".to_string(),
        ));
    }

    // Check if minimum players requirement is met
    if tournament.current_players < tournament.min_players {
        return Err(ApiError::BadRequest(format!(
            "Not enough players. Need at least {} players, currently have {}",
            tournament.min_players, tournament.current_players
        )));
    }

    // Get all participants with their submissions
    let participants = get_tournament_participants(db, tournament_id_thing.clone()).await?;

    // Filter participants who have submitted code
    let participants_with_submissions: Vec<TournamentParticipant> = participants
        .into_iter()
        .filter(|p| p.submission_id.is_some())
        .collect();

    if participants_with_submissions.is_empty() {
        return Err(ApiError::BadRequest(
            "No participants have submitted code yet".to_string(),
        ));
    }

    // Generate matches based on the match generation type
    let _matches_created = match tournament.match_generation_type {
        MatchGenerationType::AllVsAll => {
            generate_all_vs_all_matches(db, &tournament, &participants_with_submissions).await?
        }
        MatchGenerationType::RoundRobin => {
            generate_round_robin_matches(db, &tournament, &participants_with_submissions).await?
        }
        MatchGenerationType::SingleElimination => {
            generate_single_elimination_matches(db, &tournament, &participants_with_submissions)
                .await?
        }
        MatchGenerationType::DoubleElimination => {
            generate_double_elimination_matches(db, &tournament, &participants_with_submissions)
                .await?
        }
    };

    // Update tournament status
    tournament.status = TournamentStatus::Running;
    tournament.matches_generated = true;
    tournament.updated_at = Datetime::default();

    let key = (
        tournament_id_thing.tb.as_str(),
        tournament_id_thing.id.to_string(),
    );
    let updated: Option<Tournament> = db.update(key).content(tournament).await?;

    let updated_tournament =
        updated.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))?;

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

    let match_data = Match {
        id: None,
        tournament_id: tournament.id.clone(),
        game_id: tournament.game_id.clone(),
        status: MatchStatus::Pending,
        participants: vec![
            MatchParticipant {
                submission_id: submission_id_1,
                score: None,
                metadata: None,
            },
            MatchParticipant {
                submission_id: submission_id_2,
                score: None,
                metadata: None,
            },
        ],
        metadata: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: None,
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

/// Generate single elimination matches (bracket tournament)
async fn generate_single_elimination_matches(
    _db: &Database,
    _tournament: &Tournament,
    _participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    // TODO: Implement single elimination bracket generation
    // This requires more complex logic for bracket seeding
    Err(ApiError::BadRequest(
        "Single elimination not yet implemented".to_string(),
    ))
}

/// Generate double elimination matches (double bracket tournament)
async fn generate_double_elimination_matches(
    _db: &Database,
    _tournament: &Tournament,
    _participants: &[TournamentParticipant],
) -> ApiResult<usize> {
    // TODO: Implement double elimination bracket generation
    Err(ApiError::BadRequest(
        "Double elimination not yet implemented".to_string(),
    ))
}
