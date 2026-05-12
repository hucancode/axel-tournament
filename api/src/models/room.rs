use serde::Deserialize;
use serde_json::Value;
use surrealdb::types::Datetime;
use validator::Validate;

pub use axel_core::models::room::{Room, RoomStatus};

use super::{bare_key, opt_bare_key, vec_bare_key};

#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomResponse {
    pub id: Option<String>,
    pub game_id: String,
    pub host_id: String,
    pub name: String,
    pub max_players: u32,
    pub status: RoomStatus,
    pub players: Vec<String>,
    pub tournament_id: Option<String>,
    pub allowed_user_ids: Vec<String>,
    pub is_ranked: bool,
    pub winner_id: Option<String>,
    pub config: Value,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<Room> for RoomResponse {
    fn from(r: Room) -> Self {
        Self {
            id: opt_bare_key(&r.id),
            game_id: r.game_id,
            host_id: bare_key(&r.host_id),
            name: r.name,
            max_players: r.max_players,
            status: r.status,
            players: vec_bare_key(&r.players),
            tournament_id: opt_bare_key(&r.tournament_id),
            allowed_user_ids: vec_bare_key(&r.allowed_user_ids),
            is_ranked: r.is_ranked,
            winner_id: opt_bare_key(&r.winner_id),
            config: r.config,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoomRequest {
    pub game_id: String,
    #[validate(length(min = 1, max = 80, message = "Room name must be 1-80 characters"))]
    pub name: String,
    #[validate(range(min = 2, max = 16, message = "Max players must be 2-16"))]
    pub max_players: u32,
    /// Optional tournament context. Provided rooms are ranked unless the
    /// caller is making an unranked friendly room from the lobby UI.
    pub tournament_id: Option<String>,
    /// Free-form per-game configuration. Validation is the judge's job.
    pub config: Option<Value>,
}
