use crate::app_state::AppState;
use crate::auth::validate_jwt;
use crate::games::Game;
use crate::players::Player;
use crate::players::HumanPlayer;
use axum::http::StatusCode;
use axum::response::Json;
use axum::{
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub game_id: String,
    pub host_id: String,
    pub host_username: String,
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
    pub host_username: String,
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
pub struct JoinRoomRequest {
    pub player_id: String,
    pub player_username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveRoomRequest {
    pub player_id: String,
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
pub struct HistoryMessage {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

pub struct Room {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub host_id: String,
    pub host_username: String,
    pub players: Vec<Arc<HumanPlayer>>, // WebSocket connected players
    pub player_ids: Vec<String>,        // All players joined via HTTP
    pub player_usernames: HashMap<String, String>, // player_id -> username
    pub max_players: usize,
    pub status: String, // "waiting" | "playing" | "finished"
    pub human_timeout_ms: Option<u64>,
    pub message_history: Vec<HistoryMessage>,
    pub disconnected_players: HashSet<String>,
    // pub created_at: DateTime<Utc>,
}

impl Room {
    pub fn to_response(&self) -> RoomResponse {
        let players: Vec<PlayerInfo> = self
            .player_ids
            .iter()
            .map(|id| {
                let username = self.player_usernames.get(id).cloned().unwrap_or_default();
                let connected = self.players.iter().any(|p| p.player_id() == id);
                PlayerInfo {
                    id: id.clone(),
                    username,
                    connected,
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
            host_username: self.host_username.clone(),
            players,
            reconnecting: false,
        }
    }

    pub fn to_list_item(&self) -> RoomListItem {
        RoomListItem {
            id: self.id.clone(),
            name: self.name.clone(),
            game_id: self.game_id.clone(),
            host_username: self.host_username.clone(),
            current_players: self.player_ids.len(),
            max_players: self.max_players,
            status: self.status.clone(),
        }
    }
}

// ============================================================================
// Leave Result
// ============================================================================

pub enum LeaveResult {
    Left,
    HostTransferred {
        new_host_id: String,
        new_host_username: String,
    },
    RoomDeleted,
    NotInRoom,
}

// ============================================================================
// Room Manager
// ============================================================================

pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<Room>>>>>,
    pending_players: Arc<RwLock<HashMap<String, Arc<HumanPlayer>>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            pending_players: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new room with the given host
    pub async fn create_room(
        &self,
        name: String,
        game_id: String,
        host_id: String,
        host_username: String,
        max_players: usize,
        human_timeout_ms: Option<u64>,
    ) -> Room {
        let room_id = format!("room_{}", uuid::Uuid::new_v4());
        let mut player_usernames = HashMap::new();
        player_usernames.insert(host_id.clone(), host_username.clone());

        let room = Room {
            id: room_id.clone(),
            name,
            game_id,
            host_id: host_id.clone(),
            host_username,
            players: Vec::new(),
            player_ids: vec![host_id],
            player_usernames,
            max_players,
            status: "waiting".to_string(),
            human_timeout_ms,
            message_history: Vec::new(),
            disconnected_players: HashSet::new(),
            // created_at: Utc::now(),
        };

        let response = room.to_response();
        let mut rooms = self.rooms.write().await;
        rooms.insert(room_id.clone(), Arc::new(RwLock::new(room)));

        // Return the room response by creating a minimal room for response
        Room {
            id: response.id,
            name: response.name,
            game_id: response.game_id,
            host_id: response.host_id.clone(),
            host_username: response.host_username,
            players: Vec::new(),
            player_ids: vec![response.host_id],
            player_usernames: HashMap::new(),
            max_players: response.max_players as usize,
            status: response.status,
            human_timeout_ms,
            message_history: Vec::new(),
            disconnected_players: HashSet::new(),
            // created_at: Utc::now(),
        }
    }

    /// Join an existing room. Returns (room_response, is_reconnecting)
    pub async fn join_room(
        &self,
        room_id: &str,
        player_id: String,
        player_username: String,
    ) -> Result<(RoomResponse, bool), String> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;

            // Check if player is reconnecting
            if room.player_ids.contains(&player_id) {
                // Player is already in room - this is a reconnection
                let is_reconnecting = room.disconnected_players.contains(&player_id);
                let mut response = room.to_response();
                response.reconnecting = is_reconnecting;
                return Ok((response, is_reconnecting));
            }

            // Check if room is full
            if room.player_ids.len() >= room.max_players {
                return Err("Room is full".to_string());
            }

            // Check if room is accepting new players
            if room.status != "waiting" {
                return Err("Room is not accepting new players".to_string());
            }

            // Add player
            room.player_ids.push(player_id.clone());
            room.player_usernames
                .insert(player_id.clone(), player_username.clone());

            // Add to message history
            let msg = format!("PLAYER_JOINED {} {}", player_id, player_username);
            room.message_history.push(HistoryMessage {
                timestamp: Utc::now(),
                message: msg,
            });

            Ok((room.to_response(), false))
        } else {
            Err("Room not found".to_string())
        }
    }

    /// Leave a room. Handles host transfer if needed.
    pub async fn leave_room(&self, room_id: &str, player_id: &str) -> LeaveResult {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;

            // Check if player is in room
            if !room.player_ids.contains(&player_id.to_string()) {
                return LeaveResult::NotInRoom;
            }

            let username = room
                .player_usernames
                .get(player_id)
                .cloned()
                .unwrap_or_default();

            // Remove player
            room.player_ids.retain(|id| id != player_id);
            room.player_usernames.remove(player_id);
            room.players.retain(|p| p.player_id() != player_id);
            room.disconnected_players.remove(player_id);

            // Add to message history
            let msg = format!("PLAYER_LEFT {} {}", player_id, username);
            room.message_history.push(HistoryMessage {
                timestamp: Utc::now(),
                message: msg,
            });

            // Handle host transfer
            if room.host_id == player_id {
                if let Some(new_host_id) = room.player_ids.first().cloned() {
                    let new_host_username = room
                        .player_usernames
                        .get(&new_host_id)
                        .cloned()
                        .unwrap_or_default();
                    room.host_id = new_host_id.clone();
                    room.host_username = new_host_username.clone();

                    // Add host changed to history
                    let host_msg = format!("HOST_CHANGED {} {}", new_host_id, new_host_username);
                    room.message_history.push(HistoryMessage {
                        timestamp: Utc::now(),
                        message: host_msg,
                    });

                    // Broadcast to connected players
                    let broadcast_msg =
                        format!("HOST_CHANGED {} {}", new_host_id, new_host_username);
                    for player in &room.players {
                        let _ = player.send_message(&broadcast_msg).await;
                    }

                    return LeaveResult::HostTransferred {
                        new_host_id,
                        new_host_username,
                    };
                } else {
                    // Room is empty, mark for deletion
                    drop(room);
                    drop(rooms);
                    self.delete_room(room_id).await;
                    return LeaveResult::RoomDeleted;
                }
            }

            // Broadcast player left to remaining players
            let broadcast_msg = format!("PLAYER_LEFT {} {}", player_id, username);
            for player in &room.players {
                let _ = player.send_message(&broadcast_msg).await;
            }

            LeaveResult::Left
        } else {
            LeaveResult::NotInRoom
        }
    }

    /// Delete a room
    async fn delete_room(&self, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        rooms.remove(room_id);
    }

    /// List all waiting rooms, optionally filtered by game_id
    pub async fn list_rooms(&self, game_id: Option<&str>) -> Vec<RoomListItem> {
        let rooms = self.rooms.read().await;
        let mut result = Vec::new();

        for room_arc in rooms.values() {
            let room = room_arc.read().await;
            if room.status == "waiting" {
                if let Some(gid) = game_id {
                    if room.game_id != gid {
                        continue;
                    }
                }
                result.push(room.to_list_item());
            }
        }

        // Sort by created_at (newest first)
        result.sort_by(|a, b| b.id.cmp(&a.id)); // room_id contains timestamp info from UUID
        result
    }

    /// Get room info
    pub async fn get_room(&self, room_id: &str) -> Option<RoomResponse> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let room = room_arc.read().await;
            Some(room.to_response())
        } else {
            None
        }
    }

    /// Add a WebSocket player to pending list
    pub async fn add_websocket_player(&self, player_id: &str, player: Arc<HumanPlayer>) {
        let mut pending = self.pending_players.write().await;
        pending.insert(player_id.to_string(), player);
    }

    /// Connect a pending WebSocket player to a room
    /// Returns (is_reconnecting, message_history) if successful
    pub async fn connect_player_to_room(
        &self,
        room_id: &str,
        player_id: &str,
    ) -> Result<(bool, Vec<HistoryMessage>), String> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;

            // Check if player is in the room's player_ids
            if !room.player_ids.contains(&player_id.to_string()) {
                return Err("Player not in room".to_string());
            }

            // Get the WebSocket player from pending
            let pending = self.pending_players.read().await;
            if let Some(player) = pending.get(player_id) {
                // Check if reconnecting
                let is_reconnecting = room.disconnected_players.remove(player_id);

                tracing::debug!("RoomManager: Connecting player {} to room {} (reconnecting: {})", player_id, room_id, is_reconnecting);

                // Add to room's connected players
                room.players.push(Arc::clone(player));

                tracing::debug!("RoomManager: Room {} now has {} connected players", room_id, room.players.len());

                // If not reconnecting, broadcast PLAYER_JOINED to other players
                if !is_reconnecting {
                    let username = room
                        .player_usernames
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default();
                    let msg = format!("PLAYER_JOINED {} {}", player_id, username);
                    for p in &room.players {
                        if p.player_id() != player_id {
                            let _ = p.send_message(&msg).await;
                        }
                    }
                }

                let history = room.message_history.clone();
                Ok((is_reconnecting, history))
            } else {
                Err("Player WebSocket not found".to_string())
            }
        } else {
            Err("Room not found".to_string())
        }
    }

    /// Handle WebSocket disconnect - add to disconnected_players, transfer host if needed
    pub async fn on_websocket_disconnect(
        &self,
        room_id: &str,
        player_id: &str,
    ) -> Option<(String, String)> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;

            // Remove from connected players
            room.players.retain(|p| p.player_id() != player_id);

            // Add to disconnected (but keep in player_ids for reconnection)
            room.disconnected_players.insert(player_id.to_string());

            let username = room
                .player_usernames
                .get(player_id)
                .cloned()
                .unwrap_or_default();

            // Add PLAYER_LEFT to history (disconnected, not explicit leave)
            let msg = format!("PLAYER_LEFT {} {}", player_id, username);
            room.message_history.push(HistoryMessage {
                timestamp: Utc::now(),
                message: msg.clone(),
            });

            // Broadcast to remaining connected players
            for player in &room.players {
                let _ = player.send_message(&msg).await;
            }

            // Handle host disconnect - transfer to next connected player
            if room.host_id == player_id {
                // Find first connected player to be new host
                if let Some(new_host) = room.players.first() {
                    let new_host_id = new_host.player_id().to_string();
                    let new_host_username = room
                        .player_usernames
                        .get(&new_host_id)
                        .cloned()
                        .unwrap_or_default();
                    room.host_id = new_host_id.clone();
                    room.host_username = new_host_username.clone();

                    // Add to history
                    let host_msg = format!("HOST_CHANGED {} {}", new_host_id, new_host_username);
                    room.message_history.push(HistoryMessage {
                        timestamp: Utc::now(),
                        message: host_msg.clone(),
                    });

                    // Broadcast
                    for player in &room.players {
                        let _ = player.send_message(&host_msg).await;
                    }

                    return Some((new_host_id, new_host_username));
                }
            }

            None
        } else {
            None
        }
    }

    /// Broadcast a message to all connected players in a room and add to history
    pub async fn broadcast_to_room(&self, room_id: &str, message: &str, add_to_history: bool) {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;

            if add_to_history {
                room.message_history.push(HistoryMessage {
                    timestamp: Utc::now(),
                    message: message.to_string(),
                });
            }

            for player in &room.players {
                let _ = player.send_message(message).await;
            }
        }
    }

    /// Start a game in a room
    pub async fn start_game<G: Game>(
        &self,
        room_id: &str,
        host_id: &str,
        game: &G,
    ) -> Result<Vec<crate::games::GameResult>, String> {
        tracing::info!("RoomManager: Starting game in room {} by host {}", room_id, host_id);

        // Collect players and setup - release lock before running game
        let (players, human_timeout) = {
            let rooms = self.rooms.read().await;
            let room_arc = rooms.get(room_id).ok_or("Room not found")?;
            let mut room = room_arc.write().await;

            if room.host_id != host_id {
                tracing::warn!("RoomManager: Non-host {} tried to start game in room {}", host_id, room_id);
                return Err("Only host can start the game".to_string());
            }

            if room.status != "waiting" {
                tracing::warn!("RoomManager: Game already started or finished in room {} (status: {})", room_id, room.status);
                return Err("Game already started or finished".to_string());
            }

            if room.players.len() < 2 {
                tracing::warn!("RoomManager: Not enough players in room {} ({} players)", room_id, room.players.len());
                return Err("Need at least 2 connected players to start".to_string());
            }

            tracing::info!("RoomManager: Room {} has {} connected players, starting game", room_id, room.players.len());
            room.status = "playing".to_string();

            // Add GAME_STARTED to history
            room.message_history.push(HistoryMessage {
                timestamp: Utc::now(),
                message: "GAME_STARTED".to_string(),
            });

            // Broadcast to all players
            tracing::debug!("RoomManager: Broadcasting GAME_STARTED to {} players", room.players.len());
            for (i, player) in room.players.iter().enumerate() {
                tracing::debug!("RoomManager: Sending GAME_STARTED to player {} ({})", i, player.player_id());
                let _ = player.send_message("GAME_STARTED").await;
            }

            // Get timeout
            let game_metadata = crate::games::find_game_by_id(&room.game_id)
                .ok_or_else(|| format!("Game metadata not found for {}", room.game_id))?;

            let human_timeout = room
                .human_timeout_ms
                .unwrap_or(game_metadata.human_turn_timeout_ms);

            tracing::info!("RoomManager: Using timeout {}ms for room {}", human_timeout, room_id);

            let players: Vec<Box<dyn crate::players::Player>> = room
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    tracing::debug!("RoomManager: Game will use player {} with ID: {}", i, p.player_id());
                    Box::new(Arc::clone(p)) as Box<dyn crate::players::Player>
                })
                .collect();

            (players, human_timeout)
        }; // Room write lock is released here!

        // Run game without holding room lock - this allows WebSocket handlers to continue processing messages
        tracing::info!("RoomManager: Starting game execution with {} players", players.len());
        let results = game.run(players, human_timeout).await;
        tracing::info!("RoomManager: Game execution completed with results: {:?}", results);

        // Re-acquire lock to update status and broadcast results
        {
            let rooms = self.rooms.read().await;
            if let Some(room_arc) = rooms.get(room_id) {
                let mut room = room_arc.write().await;
                room.status = "finished".to_string();

                // Add GAME_FINISHED to history
                let results_json = serde_json::to_string(&results).unwrap_or_default();
                room.message_history.push(HistoryMessage {
                    timestamp: Utc::now(),
                    message: format!("GAME_FINISHED {}", results_json),
                });

                // Broadcast game finished
                tracing::debug!("RoomManager: Broadcasting GAME_FINISHED to {} players", room.players.len());
                for (i, player) in room.players.iter().enumerate() {
                    tracing::debug!("RoomManager: Sending GAME_FINISHED to player {} ({})", i, player.player_id());
                    let _ = player
                        .send_message(&format!("GAME_FINISHED {}", results_json))
                        .await;
                }

                tracing::info!("RoomManager: Game in room {} completed successfully", room_id);
            }
        }

        Ok(results)
    }

    /// Remove player from pending
    pub async fn remove_pending_player(&self, player_id: &str) {
        let mut pending = self.pending_players.write().await;
        pending.remove(player_id);
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new room
pub async fn create_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    let room = state
        .room_manager
        .create_room(
            request.name,
            request.game_id,
            request.host_id,
            request.host_username,
            state.game.max_players(),
            request.human_timeout_ms,
        )
        .await;

    Ok(Json(room.to_response()))
}

/// Get room details
pub async fn get_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    if let Some(room) = state.room_manager.get_room(&room_id).await {
        Ok(Json(room))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Join a room
pub async fn join_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
    Json(request): Json<JoinRoomRequest>,
) -> Result<Json<RoomResponse>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    match state
        .room_manager
        .join_room(&room_id, request.player_id, request.player_username)
        .await
    {
        Ok((response, _is_reconnecting)) => Ok(Json(response)),
        Err(e) => {
            tracing::warn!("Failed to join room: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Leave a room
pub async fn leave_room<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
    Json(request): Json<LeaveRoomRequest>,
) -> Result<Json<serde_json::Value>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    match state
        .room_manager
        .leave_room(&room_id, &request.player_id)
        .await
    {
        LeaveResult::Left => Ok(Json(serde_json::json!({"status": "left"}))),
        LeaveResult::HostTransferred {
            new_host_id,
            new_host_username,
        } => Ok(Json(serde_json::json!({
            "status": "left",
            "new_host_id": new_host_id,
            "new_host_username": new_host_username
        }))),
        LeaveResult::RoomDeleted => Ok(Json(serde_json::json!({"status": "room_deleted"}))),
        LeaveResult::NotInRoom => Err(StatusCode::NOT_FOUND),
    }
}

/// List all waiting rooms
pub async fn list_rooms<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Query(query): Query<ListRoomsQuery>,
) -> Json<Vec<RoomListItem>>
where
    G: Game + Clone + Send + Sync + 'static,
{
    let rooms = state
        .room_manager
        .list_rooms(query.game_id.as_deref())
        .await;
    Json(rooms)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartGameRequest {
    pub host_id: String,
    pub human_timeout_ms: Option<u64>,
}

pub async fn start_game<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    Path(room_id): Path<String>,
    Json(request): Json<StartGameRequest>,
) -> Result<Json<serde_json::Value>, StatusCode>
where
    G: Game + Clone + Send + Sync + 'static,
{
    match state
        .room_manager
        .start_game(&room_id, &request.host_id, &state.game)
        .await
    {
        Ok(results) => Ok(Json(serde_json::json!({
            "status": "completed",
            "results": results
        }))),
        Err(e) => {
            tracing::error!("Failed to start game: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn ws_get_room<G: Game + Clone + Send + Sync + 'static>(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState<G>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, room_id, state))
}

async fn handle_websocket<G: Game + Clone + Send + Sync + 'static>(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState<G>>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for first message which must be "LOGIN <jwt>"
    let player_id = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            let text = text.trim();
            if !text.starts_with("LOGIN ") {
                let _ = sender
                    .send(Message::Text("LOGIN_FAILED Missing LOGIN command".into()))
                    .await;
                return;
            }

            let token = &text[6..]; // Remove "LOGIN " prefix
            match validate_jwt(token, &state.jwt_secret) {
                Ok(user_id) => {
                    // Verify user is in room's players list
                    if let Some(room) = state.room_manager.get_room(&room_id).await {
                        let is_in_room = room.players.iter().any(|p| p.id == user_id);
                        if !is_in_room {
                            let _ = sender
                                .send(Message::Text("LOGIN_FAILED User not in room".into()))
                                .await;
                            return;
                        }
                    } else {
                        let _ = sender
                            .send(Message::Text("LOGIN_FAILED Room not found".into()))
                            .await;
                        return;
                    }
                    user_id
                }
                Err(e) => {
                    let msg = format!("LOGIN_FAILED {}", e);
                    let _ = sender.send(Message::Text(msg.into())).await;
                    return;
                }
            }
        }
        _ => {
            let _ = sender
                .send(Message::Text("LOGIN_FAILED Connection closed".into()))
                .await;
            return;
        }
    };

    // Now create the player with validated player_id
    let (move_tx, move_rx) = tokio::sync::mpsc::unbounded_channel();
    let player = Arc::new(HumanPlayer::new(player_id.clone(), sender, move_rx));

    tracing::debug!("WebSocket {}: Created new HumanPlayer instance with fresh channels", player_id);

    // Add WebSocket player to pending players
    state
        .room_manager
        .add_websocket_player(&player_id, player.clone())
        .await;

    // Try to connect player to room
    match state
        .room_manager
        .connect_player_to_room(&room_id, &player_id)
        .await
    {
        Ok((is_reconnecting, history)) => {
            if is_reconnecting {
                // Send LOGIN_OK with RECONNECT flag
                let _ = player
                    .send_message(&format!("LOGIN_OK {} RECONNECT", player_id))
                    .await;
                let _ = player.send_message("REPLAY_START").await;

                // Send room-level message history
                for msg in history {
                    let _ = player.send_message(&msg.message).await;
                }

                // Send game-level reconnection state
                let game_state = state.game.get_reconnect_state(&player_id);
                for msg in game_state {
                    let _ = player.send_message(&msg).await;
                }

                let _ = player.send_message("REPLAY_END").await;
            } else {
                // Send LOGIN_OK for new connection
                let _ = player
                    .send_message(&format!("LOGIN_OK {}", player_id))
                    .await;
            }
        }
        Err(e) => {
            let error_msg = format!("ERROR {}", e);
            let _ = player.send_message(&error_msg).await;
            return;
        }
    }

    // Handle incoming messages
    tracing::debug!("WebSocket {}: Starting message handling loop", player_id);
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text = text.trim();
                tracing::debug!("WebSocket {}: Received message: '{}'", player_id, text);

                if text == "LEAVE" {
                    tracing::info!("WebSocket {}: Player leaving room", player_id);
                    // Leave room gracefully
                    state.room_manager.leave_room(&room_id, &player_id).await;
                    let leave_msg = "LEFT_ROOM";
                    let _ = player.send_message(leave_msg).await;
                    break;
                } else if text.starts_with("CHAT ") {
                    // Chat message - broadcast to other players in room
                    let chat_content = &text[5..]; // Remove "CHAT " prefix
                    let chat_msg = format!("CHAT {} {}", player_id, chat_content);
                    tracing::debug!("WebSocket {}: Broadcasting chat message", player_id);
                    state
                        .room_manager
                        .broadcast_to_room(&room_id, &chat_msg, true)
                        .await;
                } else if text == "START" {
                    tracing::info!("WebSocket {}: Host starting game", player_id);
                    // Host is starting the game - spawn in separate task to avoid blocking WebSocket handler
                    let room_id_clone = room_id.clone();
                    let player_id_clone = player_id.clone();
                    let state_clone = state.clone();
                    let player_clone = player.clone();
                    tokio::spawn(async move {
                        match state_clone
                            .room_manager
                            .start_game(&room_id_clone, &player_id_clone, &state_clone.game)
                            .await
                        {
                            Ok(_results) => {
                                tracing::info!("WebSocket {}: Game started successfully", player_id_clone);
                                // Game results are handled by the game logic itself
                            }
                            Err(e) => {
                                tracing::error!("WebSocket {}: Failed to start game: {}", player_id_clone, e);
                                let error_msg = format!("ERROR {}", e);
                                let _ = player_clone.send_message(&error_msg).await;
                            }
                        }
                    });
                } else {
                    // Forward as game move
                    tracing::info!("WebSocket {}: Received game move from client: '{}'", player_id, text);
                    match move_tx.send(text.to_string()) {
                        Ok(_) => {
                            tracing::info!("WebSocket {}: Successfully forwarded move to game channel", player_id);
                        },
                        Err(e) => {
                            tracing::error!("WebSocket {}: Failed to forward move to channel: {}", player_id, e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket closed for player {}", player_id);
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error for player {}: {}", player_id, e);
                break;
            }
            _ => {
                tracing::debug!("WebSocket {}: Received non-text message", player_id);
            }
        }
    }
    tracing::info!("WebSocket {}: Message handling loop ended", player_id);
    // Cleanup on disconnect - keep player in room but mark as disconnected
    state
        .room_manager
        .on_websocket_disconnect(&room_id, &player_id)
        .await;
    state.room_manager.remove_pending_player(&player_id).await;
    tracing::info!("Player {} disconnected from room {}", player_id, room_id);
}
