use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::tournament::{Tournament, TournamentParticipant, TournamentStatus},
};
use std::collections::HashSet;
use surrealdb::types::{RecordId, SurrealValue};

use super::crud::get_tournament;
use super::generation;
use super::participants::get_tournament_participants;

/// Roll back a failed tournament start: delete generated matches and put
/// the tournament back into `registration`.
async fn rollback_tournament_start(db: &Database, tournament_id: &RecordId) {
    let _ = db
        .query("DELETE match WHERE tournament_id = $tournament_id")
        .bind(("tournament_id", tournament_id.clone()))
        .await;
    let _ = db
        .query(
            "UPDATE $tournament_id
             SET status = 'registration', updated_at = time::now()",
        )
        .bind(("tournament_id", tournament_id.clone()))
        .await;
}

/// Start a tournament and generate matches based on the configured match
/// generation type.
pub async fn start_tournament(
    db: &Database,
    tournament_id: RecordId,
) -> AppResult<Tournament> {
    let tournament = get_tournament(db, tournament_id.clone()).await?;
    if tournament.status != TournamentStatus::Registration {
        return Err(AppError::BadRequest(
            "Tournament must be in registration state to start".to_string(),
        ));
    }

    let participants = get_tournament_participants(db, tournament_id.clone()).await?;
    if tournament.min_players > participants.len() as u32 {
        return Err(AppError::BadRequest(format!(
            "Not enough players. Need at least {} players, currently have {}",
            tournament.min_players,
            participants.len()
        )));
    }

    // Filter to participants whose selected submission compiled cleanly.
    // One DB roundtrip via id-list, not N selects.
    let participants_with_submissions =
        compiled_participants(db, &participants).await?;

    if participants_with_submissions.is_empty() {
        return Err(AppError::BadRequest(
            "No participants with a compiled submission. Wait for compilation to finish or upload a fixed bot.".to_string(),
        ));
    }
    if (participants_with_submissions.len() as u32) < tournament.min_players {
        return Err(AppError::BadRequest(format!(
            "Not enough compiled submissions. Need {}, have {}.",
            tournament.min_players,
            participants_with_submissions.len()
        )));
    }

    // Claim tournament start to prevent duplicate match generation.
    let mut claimed = db
        .query(
            "UPDATE $tournament_id
             SET status = 'generating', updated_at = time::now()
             WHERE status = 'registration'
             RETURN AFTER",
        )
        .bind(("tournament_id", tournament_id.clone()))
        .await?;
    let claimed_rows: Vec<Tournament> = claimed.take(0)?;
    if claimed_rows.is_empty() {
        return Err(AppError::BadRequest(
            "Tournament has already been started".to_string(),
        ));
    }

    let matches_created = match generation::generate_matches(
        db,
        &tournament,
        &participants_with_submissions,
    )
    .await
    {
        Ok(count) => count,
        Err(err) => {
            rollback_tournament_start(db, &tournament_id).await;
            return Err(err);
        }
    };

    if matches_created == 0 {
        rollback_tournament_start(db, &tournament_id).await;
        return Err(AppError::Internal(
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
        .bind(("tournament_id", tournament_id.clone()))
        .await?;
    let updated_rows: Vec<Tournament> = updated.take(0)?;
    if let Some(t) = updated_rows.into_iter().next() {
        return Ok(t);
    }
    get_tournament(db, tournament_id).await
}

/// Filter `participants` to those whose selected submission compiled
/// (`status = 'accepted'`). One DB roundtrip with `WHERE id IN $ids`.
async fn compiled_participants(
    db: &Database,
    participants: &[TournamentParticipant],
) -> AppResult<Vec<TournamentParticipant>> {
    let sub_ids: Vec<RecordId> = participants
        .iter()
        .filter_map(|p| p.submission_id.clone())
        .collect();
    if sub_ids.is_empty() {
        return Ok(Vec::new());
    }
    #[derive(serde::Deserialize, SurrealValue)]
    struct AcceptedRow {
        id: RecordId,
    }
    let mut resp = db
        .query("SELECT id FROM submission WHERE id IN $ids AND status = 'accepted'")
        .bind(("ids", sub_ids))
        .await?;
    let rows: Vec<AcceptedRow> = resp.take(0)?;
    let accepted: HashSet<RecordId> = rows.into_iter().map(|r| r.id).collect();
    Ok(participants
        .iter()
        .filter(|p| {
            p.submission_id
                .as_ref()
                .is_some_and(|sid| accepted.contains(sid))
        })
        .cloned()
        .collect())
}
