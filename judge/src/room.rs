use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::game_logic::GameLogic;
use crate::players::HumanPlayer;
use crate::player::Player;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub game_id: String,
    pub host_id: String,
    pub human_timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomResponse {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub max_players: u32,
    pub current_players: u32,
    pub status: String,
    pub host_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomRequest {
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartGameRequest {
    pub host_id: String,
    pub human_timeout_ms: Option<u64>,
}

pub struct Room {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub host_id: String,
    pub players: Vec<Arc<HumanPlayer>>,
    pub player_ids: Vec<String>, // Track player IDs for HTTP operations
    pub max_players: usize,
    pub status: String,
    pub human_timeout_ms: Option<u64>,
}

pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<Room>>>>>,
    pending_players: Arc<RwLock<HashMap<String, Arc<HumanPlayer>>>>, // player_id -> HumanPlayer
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            pending_players: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_room(&self, name: String, game_id: String, host_id: String, max_players: usize, human_timeout_ms: Option<u64>) -> String {
        let room_id = format!("room_{}", uuid::Uuid::new_v4());
        let room = Room {
            id: room_id.clone(),
            name,
            game_id,
            host_id: host_id.clone(),
            players: Vec::new(),
            player_ids: vec![host_id],
            max_players,
            status: "waiting".to_string(),
            human_timeout_ms,
        };
        
        let mut rooms = self.rooms.write().await;
        rooms.insert(room_id.clone(), Arc::new(RwLock::new(room)));
        room_id
    }

    pub async fn join_room(&self, room_id: &str, player_id: String) -> Result<(), String> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            if room.player_ids.len() >= room.max_players {
                return Err("Room is full".to_string());
            }
            room.player_ids.push(player_id);
            Ok(())
        } else {
            Err("Room not found".to_string())
        }
    }

    pub async fn add_websocket_player(&self, player_id: &str, player: Arc<HumanPlayer>) {
        let mut pending = self.pending_players.write().await;
        pending.insert(player_id.to_string(), player);
    }

    pub async fn connect_player_to_room(&self, room_id: &str, player_id: &str) -> Result<(), String> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            
            // Check if player_id is in the room
            if !room.player_ids.contains(&player_id.to_string()) {
                return Err("Player not in room".to_string());
            }

            // Get the WebSocket player
            let pending = self.pending_players.read().await;
            if let Some(player) = pending.get(player_id) {
                room.players.push(Arc::clone(player));
                Ok(())
            } else {
                Err("Player WebSocket not found".to_string())
            }
        } else {
            Err("Room not found".to_string())
        }
    }

    pub async fn leave_room(&self, room_id: &str, player_id: &str) {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            room.player_ids.retain(|id| id != player_id);
            room.players.retain(|p| p.player_id() != player_id);
        }

        let mut pending = self.pending_players.write().await;
        pending.remove(player_id);
    }

    pub async fn start_game<G: GameLogic>(&self, room_id: &str, host_id: &str, game: &G) -> Result<Vec<crate::game_logic::GameResult>, String> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            
            if room.host_id != host_id {
                return Err("Only host can start the game".to_string());
            }
            
            if room.status != "waiting" {
                return Err("Game already started or finished".to_string());
            }
            
            room.status = "playing".to_string();
            
            // Get game metadata for timeouts
            let game_metadata = crate::game_metadata::find_game_by_id(&room.game_id)
                .ok_or_else(|| format!("Game metadata not found for {}", room.game_id))?;
            
            // Use room's custom human timeout or game default
            let human_timeout = room.human_timeout_ms.unwrap_or(game_metadata.human_turn_timeout_ms);
            
            let players: Vec<Box<dyn Player>> = room.players.iter()
                .map(|p| Box::new(Arc::clone(p)) as Box<dyn Player>)
                .collect();
            
            let results = game.run(players, human_timeout).await;
            room.status = "finished".to_string();
            Ok(results)
        } else {
            Err("Room not found".to_string())
        }
    }

    pub async fn get_room(&self, room_id: &str) -> Option<RoomResponse> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let room = room_arc.read().await;
            Some(RoomResponse {
                id: room.id.clone(),
                name: room.name.clone(),
                game_id: room.game_id.clone(),
                max_players: room.max_players as u32,
                current_players: room.player_ids.len() as u32,
                status: room.status.clone(),
                host_id: room.host_id.clone(),
            })
        } else {
            None
        }
    }
}

pub async fn create_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: GameLogic + Clone + Send + Sync + 'static,
{
    let room_id = state.room_manager.create_room(
        request.name.clone(),
        request.game_id.clone(),
        request.host_id.clone(),
        state.game.max_players(),
        request.human_timeout_ms
    ).await;
    
    let response = RoomResponse {
        id: room_id,
        name: request.name,
        game_id: request.game_id,
        max_players: state.game.max_players() as u32,
        current_players: 1,
        status: "waiting".to_string(),
        host_id: request.host_id,
    };
    
    Ok(Json(response))
}

pub async fn get_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: GameLogic + Clone + Send + Sync + 'static,
{
    if let Some(room) = state.room_manager.get_room(&room_id).await {
        Ok(Json(room))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn join_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
    Json(request): Json<JoinRoomRequest>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: GameLogic + Clone + Send + Sync + 'static,
{
    match state.room_manager.join_room(&room_id, request.player_id).await {
        Ok(()) => {
            if let Some(room) = state.room_manager.get_room(&room_id).await {
                Ok(Json(room))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn start_game<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
    Json(request): Json<StartGameRequest>,
) -> Result<Json<serde_json::Value>, StatusCode>
where
    G: GameLogic + Clone + Send + Sync + 'static,
{
    match state.room_manager.start_game(&room_id, &request.host_id, &state.game).await {
        Ok(results) => {
            Ok(Json(serde_json::json!({
                "status": "completed",
                "results": results
            })))
        }
        Err(e) => {
            tracing::error!("Failed to start game: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
