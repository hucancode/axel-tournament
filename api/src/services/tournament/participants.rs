use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::tournament::{TournamentParticipant, TournamentStatus},
};
use axel_core::repo::participant::ParticipantRepo;
use surrealdb::types::{Datetime, RecordId, SurrealValue};

use super::crud::get_tournament;

pub async fn join_tournament(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> AppResult<TournamentParticipant> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(AppError::BadRequest(
            "Tournament is not accepting registrations".to_string(),
        ));
    }

    let participants = get_tournament_participants(db, tournament_id.clone()).await?;
    if participants.len() as u32 >= tournament.max_players {
        return Err(AppError::BadRequest("Tournament is full".to_string()));
    }
    if participants.iter().any(|p| p.user_id == user_id) {
        return Err(AppError::Conflict(
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
    <Database as ParticipantRepo>::create(db, participant).await
}

pub async fn get_tournament_participants(
    db: &Database,
    tournament_id: RecordId,
) -> AppResult<Vec<TournamentParticipant>> {
    <Database as ParticipantRepo>::list_by_tournament(db, &tournament_id).await
}

pub async fn get_tournament_participants_with_usernames(
    db: &Database,
    tournament_id: RecordId,
) -> AppResult<Vec<(TournamentParticipant, Option<String>)>> {
    let participants = get_tournament_participants(db, tournament_id).await?;
    if participants.is_empty() {
        return Ok(Vec::new());
    }
    let user_ids: Vec<RecordId> = participants.iter().map(|p| p.user_id.clone()).collect();
    let mut response = db
        .query("SELECT id, username FROM user WHERE id IN $ids")
        .bind(("ids", user_ids))
        .await?;

    #[derive(serde::Deserialize, SurrealValue)]
    struct UserRow {
        id: RecordId,
        username: Option<String>,
    }
    let rows: Vec<UserRow> = response.take(0)?;
    let mut by_id: std::collections::HashMap<RecordId, String> = rows
        .into_iter()
        .filter_map(|r| r.username.map(|u| (r.id, u)))
        .collect();
    Ok(participants
        .into_iter()
        .map(|p| {
            let username = by_id.remove(&p.user_id);
            (p, username)
        })
        .collect())
}

/// Single participant lookup by `(tournament, user)`. None if absent.
pub async fn get_participant(
    db: &Database,
    tournament_id: &RecordId,
    user_id: &RecordId,
) -> AppResult<Option<TournamentParticipant>> {
    <Database as ParticipantRepo>::find_by_tournament_user(db, tournament_id, user_id).await
}

pub async fn leave_tournament(
    db: &Database,
    tournament_id: RecordId,
    user_id: RecordId,
) -> AppResult<()> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(AppError::BadRequest(
            "Cannot leave tournament after registration has closed".to_string(),
        ));
    }
    let participant = get_participant(db, &tournament_id, &user_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("You are not a participant in this tournament".to_string())
        })?;

    if let Some(pid) = participant.id.clone() {
        <Database as ParticipantRepo>::delete(db, &pid).await?;
    }
    Ok(())
}
