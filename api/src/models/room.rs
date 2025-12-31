use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Option<Thing>,
    pub game_id: String,
    pub host_id: Thing,
    pub name: String,
    pub max_players: u32,
    pub status: RoomStatus,
    pub players: Vec<Thing>, // user IDs
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RoomStatus {
    Waiting,
    Playing,
    Finished,
}

impl Default for RoomStatus {
    fn default() -> Self {
        RoomStatus::Waiting
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRoomRequest {
    pub game_id: String,
    #[validate(length(min = 1, max = 100, message = "Room name must be 1-100 characters"))]
    pub name: String,
    #[validate(range(min = 2, max = 8, message = "Max players must be 2-8"))]
    pub max_players: u32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoomRequest {
    #[validate(length(min = 1, max = 100, message = "Room name must be 1-100 characters"))]
    pub name: Option<String>,
    #[validate(range(min = 2, max = 8, message = "Max players must be 2-8"))]
    pub max_players: Option<u32>,
    pub status: Option<RoomStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomResponse {
    pub id: String,
    pub game_id: String,
    pub host_id: String,
    pub name: String,
    pub max_players: u32,
    pub status: RoomStatus,
    pub players: Vec<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<Room> for RoomResponse {
    fn from(room: Room) -> Self {
        Self {
            id: room.id.map(|t| t.to_string()).unwrap_or_default(),
            game_id: room.game_id,
            host_id: room.host_id.to_string(),
            name: room.name,
            max_players: room.max_players,
            status: room.status,
            players: room.players.into_iter().map(|p| p.to_string()).collect(),
            created_at: room.created_at,
            updated_at: room.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMessage {
    pub id: Option<Thing>,
    pub room_id: Thing,
    pub user_id: Thing,
    pub message: String,
    pub created_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRoomMessageRequest {
    #[validate(length(min = 1, max = 500, message = "Message must be 1-500 characters"))]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomMessageResponse {
    pub id: String,
    pub room_id: String,
    pub user_id: String,
    pub message: String,
    pub created_at: Datetime,
}

impl From<RoomMessage> for RoomMessageResponse {
    fn from(msg: RoomMessage) -> Self {
        Self {
            id: msg.id.map(|t| t.to_string()).unwrap_or_default(),
            room_id: msg.room_id.to_string(),
            user_id: msg.user_id.to_string(),
            message: msg.message,
            created_at: msg.created_at,
        }
    }
}
