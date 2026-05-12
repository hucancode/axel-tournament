use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{
        game::find_game_by_id,
        matches::{Match, MatchParticipant},
    },
};
use axel_core::repo::matches::{MatchListFilter, MatchRepo};
use axel_core::repo::submission::SubmissionRepo;
use surrealdb::types::{RecordId, ToSql};

pub async fn create_match(
    db: &Database,
    tournament_id: RecordId,
    game_id: String,
    participant_submission_ids: Vec<RecordId>,
) -> AppResult<Match> {
    find_game_by_id(&game_id)
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))?;

    let mut participants = Vec::new();
    for sub_id in participant_submission_ids {
        let submission = <Database as SubmissionRepo>::get_by_id(db, &sub_id).await?;
        if submission.game_id != game_id {
            return Err(AppError::BadRequest(format!(
                "Submission {} does not belong to game {}",
                sub_id.to_sql(),
                game_id
            )));
        }
        participants.push(MatchParticipant {
            user_id: submission.user_id.clone(),
            submission_id: submission.id.clone(),
            score: None,
        });
    }

    let new_match = Match {
        tournament_id: Some(tournament_id),
        game_id: game_id.clone(),
        participants,
        ..Default::default()
    };

    <Database as MatchRepo>::create(db, new_match).await
}

pub async fn get_match(db: &Database, match_id: RecordId) -> AppResult<Match> {
    <Database as MatchRepo>::get_by_id(db, &match_id).await
}

/// List matches with optional filters. Pagination happens DB-side.
/// `user_id` filter joins via `submission.user_id` on the SurrealQL side
/// so `LIMIT` / `START` apply to the joined rows, not a post-fetch filter.
pub async fn list_matches(
    db: &Database,
    tournament_id: Option<RecordId>,
    game_id: Option<RecordId>,
    user_id: Option<RecordId>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> AppResult<Vec<Match>> {
    <Database as MatchRepo>::list(
        db,
        MatchListFilter {
            tournament_id,
            game_id,
            user_id,
            limit,
            offset,
        },
    )
    .await
}
