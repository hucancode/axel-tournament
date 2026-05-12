use async_trait::async_trait;
use surrealdb::types::RecordId;
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::tournament::TournamentParticipant;

#[async_trait]
pub trait ParticipantRepo: Send + Sync {
    async fn create(&self, p: TournamentParticipant) -> AppResult<TournamentParticipant>;
    async fn delete(&self, participant_id: &RecordId) -> AppResult<()>;
    async fn list_by_tournament(
        &self,
        tournament_id: &RecordId,
    ) -> AppResult<Vec<TournamentParticipant>>;
    async fn find_by_tournament_user(
        &self,
        tournament_id: &RecordId,
        user_id: &RecordId,
    ) -> AppResult<Option<TournamentParticipant>>;
}

#[async_trait]
impl<C: Connection> ParticipantRepo for Surreal<C> {
    async fn create(&self, p: TournamentParticipant) -> AppResult<TournamentParticipant> {
        let created: Option<TournamentParticipant> =
            self.create("tournament_participant").content(p).await?;
        created.ok_or_else(|| AppError::Internal("Failed to create participant".into()))
    }

    async fn delete(&self, participant_id: &RecordId) -> AppResult<()> {
        let _: Option<TournamentParticipant> = self.delete(participant_id).await?;
        Ok(())
    }

    async fn list_by_tournament(
        &self,
        tournament_id: &RecordId,
    ) -> AppResult<Vec<TournamentParticipant>> {
        let mut result = self
            .query(
                "SELECT * FROM tournament_participant
                 WHERE tournament_id = $tournament_id ORDER BY score DESC",
            )
            .bind(("tournament_id", tournament_id.clone()))
            .await?;
        Ok(result.take(0)?)
    }

    async fn find_by_tournament_user(
        &self,
        tournament_id: &RecordId,
        user_id: &RecordId,
    ) -> AppResult<Option<TournamentParticipant>> {
        let mut resp = self
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
}
