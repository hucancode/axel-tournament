use async_trait::async_trait;
use surrealdb::types::RecordId;
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::room::Room;

#[async_trait]
pub trait RoomRepo: Send + Sync {
    async fn get_by_id(&self, room_id: &RecordId) -> AppResult<Room>;
    async fn find_by_id(&self, room_id: &RecordId) -> AppResult<Option<Room>>;
    async fn create(&self, room: Room) -> AppResult<Room>;
    async fn list_open(&self, game_id: Option<&str>) -> AppResult<Vec<Room>>;
}

#[async_trait]
impl<C: Connection> RoomRepo for Surreal<C> {
    async fn get_by_id(&self, room_id: &RecordId) -> AppResult<Room> {
        <Self as RoomRepo>::find_by_id(self, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))
    }

    async fn find_by_id(&self, room_id: &RecordId) -> AppResult<Option<Room>> {
        let r: Option<Room> = self.select(room_id).await?;
        Ok(r)
    }

    async fn create(&self, room: Room) -> AppResult<Room> {
        let created: Option<Room> = self.create("room").content(room).await?;
        created.ok_or_else(|| AppError::Internal("Failed to create room".into()))
    }

    async fn list_open(&self, game_id: Option<&str>) -> AppResult<Vec<Room>> {
        let mut resp = if let Some(gid) = game_id {
            self.query(
                "SELECT * FROM room WHERE status = 'lobby' AND game_id = $game_id
                 ORDER BY created_at DESC LIMIT 200",
            )
            .bind(("game_id", gid.to_string()))
            .await?
        } else {
            self.query(
                "SELECT * FROM room WHERE status = 'lobby'
                 ORDER BY created_at DESC LIMIT 200",
            )
            .await?
        };
        Ok(resp.take(0)?)
    }
}
