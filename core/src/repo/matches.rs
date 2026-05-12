use async_trait::async_trait;
use surrealdb::types::RecordId;
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::matches::Match;

pub struct MatchListFilter {
    pub tournament_id: Option<RecordId>,
    pub game_id: Option<RecordId>,
    pub user_id: Option<RecordId>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[async_trait]
pub trait MatchRepo: Send + Sync {
    async fn get_by_id(&self, match_id: &RecordId) -> AppResult<Match>;
    async fn create(&self, m: Match) -> AppResult<Match>;
    async fn list(&self, filter: MatchListFilter) -> AppResult<Vec<Match>>;
}

#[async_trait]
impl<C: Connection> MatchRepo for Surreal<C> {
    async fn get_by_id(&self, match_id: &RecordId) -> AppResult<Match> {
        let m: Option<Match> = self.select(match_id).await?;
        m.ok_or_else(|| AppError::NotFound("Match not found".into()))
    }

    async fn create(&self, m: Match) -> AppResult<Match> {
        let created: Option<Match> = self.create("match").content(m).await?;
        created.ok_or_else(|| AppError::Internal("Failed to create match".into()))
    }

    async fn list(&self, filter: MatchListFilter) -> AppResult<Vec<Match>> {
        let limit = filter.limit.unwrap_or(50).min(200);
        let offset = filter.offset.unwrap_or(0);

        let mut where_parts: Vec<&str> = Vec::new();
        if filter.tournament_id.is_some() {
            where_parts.push("tournament_id = $tournament_id");
        }
        if filter.game_id.is_some() {
            where_parts.push("game_id = $game_id");
        }
        if filter.user_id.is_some() {
            where_parts.push(
                "participants[*].submission_id ANYINSIDE (SELECT VALUE id FROM submission WHERE user_id = $user_id)",
            );
        }
        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };
        let query = format!(
            "SELECT * FROM match{} ORDER BY created_at DESC LIMIT $limit START $offset",
            where_clause
        );

        let mut q = self
            .query(query)
            .bind(("limit", limit))
            .bind(("offset", offset));
        if let Some(tid) = filter.tournament_id {
            q = q.bind(("tournament_id", tid));
        }
        if let Some(gid) = filter.game_id {
            q = q.bind(("game_id", gid));
        }
        if let Some(uid) = filter.user_id {
            q = q.bind(("user_id", uid));
        }
        let mut result = q.await?;
        Ok(result.take(0)?)
    }
}
