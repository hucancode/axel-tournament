use async_trait::async_trait;
use surrealdb::types::RecordId;
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::tournament::Tournament;

#[async_trait]
pub trait TournamentRepo: Send + Sync {
    async fn get_by_id(&self, tournament_id: &RecordId) -> AppResult<Tournament>;
    async fn find_by_id(&self, tournament_id: &RecordId) -> AppResult<Option<Tournament>>;
    async fn create(&self, tournament: Tournament) -> AppResult<Tournament>;
    async fn replace(&self, tournament_id: &RecordId, tournament: Tournament) -> AppResult<Tournament>;
}

#[async_trait]
impl<C: Connection> TournamentRepo for Surreal<C> {
    async fn get_by_id(&self, tournament_id: &RecordId) -> AppResult<Tournament> {
        <Self as TournamentRepo>::find_by_id(self, tournament_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Tournament not found".into()))
    }

    async fn find_by_id(&self, tournament_id: &RecordId) -> AppResult<Option<Tournament>> {
        let t: Option<Tournament> = self.select(tournament_id).await?;
        Ok(t)
    }

    async fn create(&self, tournament: Tournament) -> AppResult<Tournament> {
        let created: Option<Tournament> = self.create("tournament").content(tournament).await?;
        created.ok_or_else(|| AppError::Internal("Failed to create tournament".into()))
    }

    async fn replace(&self, tournament_id: &RecordId, tournament: Tournament) -> AppResult<Tournament> {
        let updated: Option<Tournament> = self.update(tournament_id).content(tournament).await?;
        updated.ok_or_else(|| AppError::NotFound("Tournament not found".into()))
    }
}
