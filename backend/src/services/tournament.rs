use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Tournament, TournamentParticipant, TournamentStatus},
};
use chrono::{DateTime, Utc};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_tournament(
    db: &Database,
    game_id: String,
    name: String,
    description: String,
    min_players: u32,
    max_players: u32,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> ApiResult<Tournament> {
    let game_id_clean = game_id.trim_start_matches("game:").to_string();
    let game_thing = Thing::from(("game", game_id_clean.as_str()));
    let tournament = Tournament {
        id: None,
        game_id: game_thing,
        name,
        description,
        status: TournamentStatus::Registration,
        min_players,
        max_players,
        current_players: 0,
        start_time: start_time.map(|dt| dt.into()),
        end_time: end_time.map(|dt| dt.into()),
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Tournament> = db.create("tournament").content(tournament).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create tournament".to_string()))
}

pub async fn get_tournament(db: &Database, tournament_id: &str) -> ApiResult<Tournament> {
    let tournament: Option<Tournament> = db.select(("tournament", tournament_id)).await?;
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
    tournament_id: &str,
    name: Option<String>,
    description: Option<String>,
    status: Option<TournamentStatus>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> ApiResult<Tournament> {
    let mut tournament = get_tournament(db, tournament_id).await?;
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
    let updated: Option<Tournament> = db
        .update(("tournament", tournament_id))
        .content(tournament)
        .await?;
    updated.ok_or_else(|| ApiError::NotFound("Tournament not found".to_string()))
}

pub async fn join_tournament(
    db: &Database,
    tournament_id: &str,
    user_id: &str,
) -> ApiResult<TournamentParticipant> {
    let user_id_clean = user_id.trim_start_matches("user:");
    let tournament_id_clean = tournament_id.trim_start_matches("tournament:");
    let tournament = get_tournament(db, tournament_id_clean).await?;
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
        .bind(("tournament_id", Thing::from(("tournament", tournament_id_clean))))
        .bind(("user_id", Thing::from(("user", user_id_clean))))
        .await?;
    let existing_participants: Vec<TournamentParticipant> = existing.take(0)?;
    if !existing_participants.is_empty() {
        return Err(ApiError::Conflict(
            "User already joined this tournament".to_string(),
        ));
    }
    let participant = TournamentParticipant {
        id: None,
        tournament_id: Thing::from(("tournament", tournament_id_clean)),
        user_id: Thing::from(("user", user_id_clean)),
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
        .bind((
            "tournament_id",
            Thing::from(("tournament", tournament_id_clean)),
        ))
        .await?;
    created.ok_or_else(|| ApiError::Internal("Failed to join tournament".to_string()))
}

pub async fn get_tournament_participants(
    db: &Database,
    tournament_id: &str,
) -> ApiResult<Vec<TournamentParticipant>> {
    let mut result = db
        .query("SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id ORDER BY score DESC")
        .bind(("tournament_id", Thing::from(("tournament", tournament_id))))
        .await?;
    let participants: Vec<TournamentParticipant> = result.take(0)?;
    Ok(participants)
}

pub async fn leave_tournament(db: &Database, tournament_id: &str, user_id: &str) -> ApiResult<()> {
    let user_id_clean = user_id.trim_start_matches("user:");
    let tournament_id_clean = tournament_id.trim_start_matches("tournament:");
    // Find participant first to avoid matching issues
    let mut existing = db
        .query("SELECT * FROM tournament_participant WHERE tournament_id = $tournament_id AND user_id = $user_id")
        .bind(("tournament_id", Thing::from(("tournament", tournament_id_clean))))
        .bind(("user_id", Thing::from(("user", user_id_clean))))
        .await?;
    let participants: Vec<TournamentParticipant> = existing.take(0)?;
    let participant = participants.into_iter().next().ok_or_else(|| {
        ApiError::NotFound("Participant not found in this tournament".to_string())
    })?;
    // Delete by specific participant id
    if let Some(pid) = participant.id.clone() {
        let delete_key = (pid.tb.as_str(), pid.id.to_string());
        let _: Option<TournamentParticipant> = db.delete(delete_key).await?;
    }
    // Decrement current_players
    db.query("UPDATE $tournament_id SET current_players -= 1")
        .bind((
            "tournament_id",
            Thing::from(("tournament", tournament_id_clean)),
        ))
        .await?;
    Ok(())
}
