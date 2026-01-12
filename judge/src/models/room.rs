use crate::db::Database;
use crate::models::players::{HumanPlayer, Player};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::{Datetime, Thing};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub game_id: String,
    pub host_id: String,
    pub human_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomResponse {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub max_players: u32,
    pub status: String,
    pub host_id: String,
    pub players: Vec<PlayerInfo>,
    pub reconnecting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub username: String,
    pub connected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRoomsQuery {
    pub game_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomListItem {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub host_username: String,
    pub current_players: usize,
    pub max_players: usize,
    pub status: String,
}

// ============================================================================
// Room and History Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    pub id: Option<Thing>,
    pub tournament_id: Option<Thing>,
    pub game_id: String,
    pub status: String,
    pub participants: Vec<MatchParticipant>,
    pub metadata: Option<serde_json::Value>,
    pub room_id: Option<Thing>,
    pub game_event_source: Option<String>,
    pub judge_server_name: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub started_at: Option<Datetime>,
    pub completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchParticipant {
    pub user_id: Thing,
    pub submission_id: Option<Thing>,
    pub score: Option<f64>,
}

/// Context for games to write events to match history
#[derive(Clone)]
pub struct GameContext {
    match_id: Thing,
    db: Database,
}

impl GameContext {
    pub fn new(match_id: Thing, db: Database) -> Self {
        Self { match_id, db }
    }

    /// Write a game event to the match history
    pub async fn write_event(&self, event: &str) {
        let query = "UPDATE $match_id SET game_event_source = string::concat(game_event_source, $event, '\n'), updated_at = time::now()";

        if let Err(e) = self
            .db
            .query(query)
            .bind(("match_id", self.match_id.clone()))
            .bind(("event", event.to_string()))
            .await
        {
            tracing::error!("Failed to write game event: {}", e);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomRecord {
    pub id: Option<Thing>,
    pub game_id: String,
    pub host_id: Thing,
    pub name: String,
    pub max_players: u32,
    pub status: String,
    pub players: Vec<Thing>,
    pub human_timeout_ms: Option<u64>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub event_history: Vec<String>,
}

#[derive(Clone)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub host_id: String,
    pub players: Vec<Thing>,            // All players who have joined the room
    pub connected_players: Vec<Option<Arc<HumanPlayer>>>, // WebSocket connections (None if offline)
    pub max_players: usize,
    pub status: String, // "waiting" | "playing" | "finished"
    pub human_timeout_ms: Option<u64>,
    pub message_history: Vec<String>,
}

impl Room {
    pub fn to_response(&self) -> RoomResponse {
        let players: Vec<PlayerInfo> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, id)| {
                let id_str = id.to_string();
                let username = id_str.strip_prefix("user:").unwrap_or(&id_str).to_string();
                let connected = self.connected_players.get(i).and_then(|p| p.as_ref()).is_some();

                if connected {
                    Some(PlayerInfo { id: id_str, username, connected: true })
                } else {
                    None
                }
            })
            .collect();

        RoomResponse {
            id: self.id.clone(),
            name: self.name.clone(),
            game_id: self.game_id.clone(),
            max_players: self.max_players as u32,
            status: self.status.clone(),
            host_id: self.host_id.clone(),
            players,
            reconnecting: false,
        }
    }

    /// Create Room from RoomRecord
    pub fn from_record(record: RoomRecord) -> Self {
        let room_id = record.id.as_ref().map(|t| t.to_string())
            .unwrap_or_else(|| format!("room_{}", uuid::Uuid::new_v4()));
        let connected_players = vec![None; record.players.len()];

        Room {
            id: room_id,
            name: record.name,
            game_id: record.game_id,
            host_id: record.host_id.to_string(),
            players: record.players,
            connected_players,
            max_players: record.max_players as usize,
            status: record.status,
            human_timeout_ms: record.human_timeout_ms,
            message_history: record.event_history,
        }
    }

    /// Broadcast message to all connected players
    pub async fn broadcast(&self, message: &str) {
        for player in self.connected_players.iter().flatten() {
            let _ = player.send_message(message).await;
        }
    }

    /// Count of currently connected players
    pub fn connected_count(&self) -> usize {
        self.connected_players.iter().filter(|p| p.is_some()).count()
    }

    /// Transfer host to next connected player. Returns new host ID if transferred.
    pub async fn transfer_host_if_needed(&mut self, leaving_player_id: &str) -> Option<String> {
        if self.host_id != leaving_player_id {
            return None;
        }

        let new_host = self.connected_players.iter()
            .flatten()
            .next()?;

        let new_host_id = new_host.player_id().to_string();
        self.host_id = new_host_id.clone();

        let msg = format!("HOST_CHANGED {}", new_host_id);
        self.message_history.push(msg.clone());
        self.broadcast(&msg).await;

        Some(new_host_id)
    }
}

// ============================================================================
// Leave Result
// ============================================================================

pub enum LeaveResult {
    Left,
    HostTransferred,
    RoomDeleted,
    NotInRoom,
}
