// RoomManager - Core orchestration layer for room lifecycle and game execution
// Manages in-memory room state, lazy loading from database, and crash recovery

use super::db;
use crate::models::room::*;
use crate::db::Database;
use crate::models::game::Game;
use crate::models::players::{HumanPlayer, Player};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use surrealdb::sql::Thing;
use tokio::sync::RwLock;

pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<Room>>>>>,
    pending_players: Arc<RwLock<HashMap<String, Arc<HumanPlayer>>>>,
    db: Database,
    restoring_rooms: Arc<RwLock<HashSet<String>>>, // Guard to prevent concurrent room restorations
}

impl RoomManager {
    pub fn new(db: Database) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            pending_players: Arc::new(RwLock::new(HashMap::new())),
            db,
            restoring_rooms: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create a new room with the given host
    pub async fn create_room(
        &self,
        name: String,
        game_id: String,
        host_id: String,
        max_players: usize,
        human_timeout_ms: Option<u64>,
    ) -> Result<Room, String> {
        tracing::info!(
            "CREATE_ROOM_START name='{}' game_id={} host_id={} max_players={}",
            name,
            game_id,
            host_id,
            max_players
        );

        // First create the room in the database
        let host_thing: Thing = match host_id.parse() {
            Ok(thing) => thing,
            Err(_) => return Err("Invalid host_id format".to_string()),
        };
        let room_record = db::create_room(
            &self.db,
            game_id.clone(),
            host_thing.clone(),
            name.clone(),
            max_players as u32,
            human_timeout_ms,
        )
        .await
        .map_err(|e| {
            tracing::error!("CREATE_ROOM_DB_FAILED error={}", e);
            format!("Failed to create room in database: {}", e)
        })?;

        let room_id = room_record
            .id
            .as_ref()
            .map(|t| t.to_string())
            .ok_or_else(|| "Database did not return room ID".to_string())?;

        tracing::info!(
            "ROOM_CREATED_DB room_id={} name='{}' game_id={} host_id={} max_players={}",
            room_id,
            name,
            game_id,
            host_id,
            max_players
        );

        // Create in-memory room
        let room = Room {
            id: room_id.clone(),
            name,
            game_id,
            host_id: host_id.clone(),
            players: vec![host_thing],
            connected_players: vec![None], // Host is in room but not yet connected via WebSocket
            max_players,
            status: "waiting".to_string(),
            human_timeout_ms,
            message_history: Vec::new(),
        };

        // Store in-memory room
        let mut rooms = self.rooms.write().await;
        rooms.insert(room_id.clone(), Arc::new(RwLock::new(room.clone())));

        tracing::info!(
            "ROOM_CREATED_MEMORY room_id={} total_rooms={}",
            room_id,
            rooms.len()
        );

        Ok(room)
    }

    /// Join an existing room. Returns (room_response, is_reconnecting)
    pub async fn join_room(
        &self,
        room_id: &str,
        player_id: String,
    ) -> Result<(RoomResponse, bool), String> {
        // First check if room exists in memory
        let rooms_read = self.rooms.read().await;
        let room_exists_in_memory = rooms_read.contains_key(room_id);
        drop(rooms_read);

        // If room doesn't exist in memory, try to load from database and create it
        if !room_exists_in_memory {
            tracing::info!(
                "JOIN_ROOM room {} not in memory, loading from database",
                room_id
            );

            // Load room from database
            let room_record = match db::get_room(&self.db, room_id).await {
                Ok(Some(record)) => record,
                Ok(None) => return Err("Room not found".to_string()),
                Err(e) => return Err(format!("Database error: {}", e)),
            };
            // Create in-memory room from database record
            let room = Room::from_record(room_record);
            // Store in memory
            let mut rooms_write = self.rooms.write().await;
            rooms_write.insert(room_id.to_string(), Arc::new(RwLock::new(room)));
            drop(rooms_write);

            tracing::info!("JOIN_ROOM created in-memory room {} from database", room_id);
        }

        // Now join the room
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            // Check if player is reconnecting
            let player_thing: Thing = player_id
                .parse()
                .map_err(|_| "Invalid player_id format".to_string())?;

            if let Some(player_index) = room.players.iter().position(|p| p == &player_thing) {
                // Player is already in room - this is a reconnection
                let is_reconnecting = room
                    .connected_players
                    .get(player_index)
                    .and_then(|p| p.as_ref())
                    .is_none();
                let mut response = room.to_response();
                response.reconnecting = is_reconnecting;
                return Ok((response, is_reconnecting));
            }

            // Check if room is full (count only online players)
            let active_players = room
                .connected_players
                .iter()
                .filter(|p| p.is_some())
                .count();
            tracing::debug!(
                "Room capacity check: active_players={}, max_players={}, players={:?}",
                active_players,
                room.max_players,
                room.players
            );
            if active_players >= room.max_players {
                tracing::warn!(
                    "Room is full: active_players={} >= max_players={}, players={:?}",
                    active_players,
                    room.max_players,
                    room.players
                );
                return Err("Room is full".to_string());
            }

            // Check if room is accepting new players
            if room.status != "waiting" {
                return Err("Room is not accepting new players".to_string());
            }

            // Add player
            let player_thing: Thing = player_id
                .parse()
                .map_err(|_| "Invalid player_id format".to_string())?;
            room.players.push(player_thing.clone());
            room.connected_players.push(None); // Player starts offline until WebSocket connects

            // Sync with database - add player to room
            if let Err(e) = db::add_player(&self.db, room_id, player_thing).await {
                tracing::error!("Failed to sync player join to database: {}", e);
                // Continue anyway - in-memory state is updated
            }

            // Don't add PLAYER_JOINED to message_history yet - will be added when WebSocket connects
            // This allows us to distinguish new players from reconnecting players

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
            let player_thing: Thing = match player_id.parse() {
                Ok(thing) => thing,
                Err(_) => return LeaveResult::NotInRoom,
            };

            let player_index = match room.players.iter().position(|p| p == &player_thing) {
                Some(index) => index,
                None => return LeaveResult::NotInRoom,
            };

            // Remove player from all arrays
            room.players.remove(player_index);
            room.connected_players.remove(player_index);

            // Sync with database - remove player from room
            if let Err(e) = db::remove_player(&self.db, room_id, player_thing.clone()).await {
                tracing::error!("Failed to sync player leave to database: {}", e);
                // Continue anyway - in-memory state is updated
            }

            // Add to message history
            let msg = format!("PLAYER_LEFT {}", player_id);
            room.message_history.push(msg);

            // Handle host transfer
            if room.host_id == player_id {
                if let Some(new_host_id) = room.players.first().cloned() {
                    room.host_id = new_host_id.to_string();
                    // Sync host change with database
                    let mut updates = HashMap::new();
                    updates.insert(
                        "host_id".to_string(),
                        serde_json::Value::String(format!("user:{}", new_host_id)),
                    );
                    if let Err(e) = db::update_room(&self.db, room_id, updates).await {
                        tracing::error!("Failed to sync host transfer to database: {}", e);
                    }

                    // Add host changed to history
                    let host_msg = format!("HOST_CHANGED {}", new_host_id);
                    room.message_history.push(host_msg);

                    // Broadcast to connected players
                    let broadcast_msg = format!("HOST_CHANGED {}", new_host_id);
                    for player in &room.connected_players {
                        if let Some(p) = player {
                            let _ = p.send_message(&broadcast_msg).await;
                        }
                    }

                    return LeaveResult::HostTransferred;
                } else {
                    // Room is empty, mark for deletion
                    drop(room);
                    drop(rooms);
                    self.delete_room(room_id).await;
                    return LeaveResult::RoomDeleted;
                }
            }

            // Broadcast player left to remaining players
            let broadcast_msg = format!("PLAYER_LEFT {}", player_id);
            for player in &room.connected_players {
                if let Some(p) = player {
                    let _ = p.send_message(&broadcast_msg).await;
                }
            }

            LeaveResult::Left
        } else {
            LeaveResult::NotInRoom
        }
    }

    /// Delete a room
    async fn delete_room(&self, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        if let Some(_) = rooms.remove(room_id) {
            tracing::info!("ROOM_CLOSED room_id={} reason=empty", room_id);

            // Also delete from database
            if let Err(e) = db::delete_room(&self.db, room_id).await {
                tracing::error!("Failed to delete room from database: {}", e);
            }
        }
    }

    /// List all waiting rooms, optionally filtered by game_id
    pub async fn list_rooms(&self, game_id: Option<&str>) -> Vec<RoomListItem> {
        match db::list_rooms(&self.db, game_id, Some("waiting")).await {
            Ok(room_records) => {
                let mut result = Vec::new();
                let rooms = self.rooms.read().await;

                for record in room_records {
                    let room_id = record
                        .id
                        .as_ref()
                        .map(|t| t.to_string())
                        .unwrap_or_default();

                    // Check if room exists in memory for real-time data
                    let (current_players, status) = if let Some(room_arc) = rooms.get(&room_id) {
                        let room = room_arc.read().await;
                        (room.players.len(), room.status.clone()) // Only count connected players
                    } else {
                        // Room not in memory, use database data but assume 0 connected
                        (0, record.status.clone())
                    };

                    result.push(RoomListItem {
                        id: room_id,
                        name: record.name,
                        game_id: record.game_id,
                        host_username: "Unknown".to_string(), // TODO: Get from user record
                        current_players,
                        max_players: record.max_players as usize,
                        status,
                    });
                }
                result
            }
            Err(e) => {
                tracing::error!("Failed to list rooms from database: {}", e);
                Vec::new()
            }
        }
    }

    /// Get room info
    pub async fn get_room(&self, room_id: &str) -> Option<RoomResponse> {
        tracing::info!("GET_ROOM_ATTEMPT room_id={}", room_id);
        // First try to get from in-memory rooms (most up-to-date)
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let room = room_arc.read().await;
            tracing::info!("GET_ROOM_SUCCESS_MEMORY room_id={}", room_id);
            return Some(room.to_response());
        }
        drop(rooms);

        // Fall back to database
        match db::get_room(&self.db, room_id).await {
            Ok(Some(record)) => {
                tracing::info!("GET_ROOM_SUCCESS_DB room_id={}", room_id);
                let players: Vec<PlayerInfo> = record
                    .players
                    .iter()
                    .map(|user_thing| {
                        PlayerInfo {
                            id: user_thing.to_string(),
                            username: "Unknown".to_string(), // TODO: Get from user record
                            connected: false, // We don't know connection status from database
                        }
                    })
                    .collect();
                let room_id_str = record
                    .id
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_default();
                Some(RoomResponse {
                    id: room_id_str,
                    name: record.name,
                    game_id: record.game_id,
                    max_players: record.max_players,
                    status: record.status,
                    host_id: record.host_id.to_string(),
                    players,
                    reconnecting: false,
                })
            }
            Ok(None) => {
                tracing::warn!("GET_ROOM_NOT_FOUND room_id={}", room_id);
                None
            }
            Err(e) => {
                tracing::error!("GET_ROOM_ERROR room_id={} error={}", room_id, e);
                None
            }
        }
    }

    /// Add a WebSocket player to pending list
    pub async fn add_websocket_player(&self, player_id: &str, player: Arc<HumanPlayer>) {
        let mut pending = self.pending_players.write().await;
        pending.insert(player_id.to_string(), player);
    }

    /// Connect a pending WebSocket player to a room
    /// Returns is_reconnecting if successful
    /// Creates in-memory room on-demand if it doesn't exist
    pub async fn connect_player_to_room(
        &self,
        room_id: &str,
        player_id: &str,
    ) -> Result<bool, String> {
        // First check if room exists in memory
        let rooms_read = self.rooms.read().await;
        let room_exists_in_memory = rooms_read.contains_key(room_id);
        drop(rooms_read);

        // If room doesn't exist in memory, try to load from database and restore it
        if !room_exists_in_memory {
            // Check if another task is already restoring this room
            let restoring = self.restoring_rooms.read().await;
            if restoring.contains(room_id) {
                // Wait for restoration to complete
                drop(restoring);
                self.wait_for_restoration(room_id).await?;

                // Verify room was successfully restored
                let rooms = self.rooms.read().await;
                if !rooms.contains_key(room_id) {
                    return Err("Room restoration failed".to_string());
                }
                drop(rooms);

                // Room is now available, continue to connection logic
            } else {
                drop(restoring);

                // Mark as restoring
                {
                    let mut restoring = self.restoring_rooms.write().await;
                    restoring.insert(room_id.to_string());
                }

                tracing::info!("ROOM_LOAD_START room_id={}", room_id);

                // Load room from database
                let result = async {
                    let room_record = db::get_room(&self.db, room_id)
                        .await
                        .map_err(|e| format!("Database error: {}", e))?
                        .ok_or_else(|| "Room not found".to_string())?;

                    // Restore room from record
                    let room = self.restore_room_from_record(room_record).await?;

                    // Store in memory
                    let mut rooms_write = self.rooms.write().await;
                    rooms_write.insert(room_id.to_string(), Arc::new(RwLock::new(room)));

                    tracing::info!("ROOM_LOAD_COMPLETE room_id={}", room_id);
                    Ok::<(), String>(())
                }
                .await;

                // Unmark as restoring
                {
                    let mut restoring = self.restoring_rooms.write().await;
                    restoring.remove(room_id);
                }

                // Propagate error if restoration failed
                result?;
            }
        }

        // Now connect the player to the room
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            let player_thing = player_id.parse::<Thing>().unwrap();
            tracing::info!(
                "Checking player {} (as Thing: {}) in room {} with players: {:?}",
                player_id,
                player_thing,
                room_id,
                room.players
            );

            let player_index = match room.players.iter().position(|p| p == &player_thing) {
                Some(index) => index,
                None => {
                    tracing::warn!(
                        "Player {} not found in room {} players: {:?}",
                        player_id,
                        room_id,
                        room.players
                    );
                    return Err("Player not in room".to_string());
                }
            };
            // Get the WebSocket player from pending
            let pending = self.pending_players.read().await;
            if let Some(player) = pending.get(player_id) {
                // Check if this is a reconnection by looking at message history
                // If PLAYER_JOINED for this player is already in history, it's a reconnection
                let player_joined_msg = format!("PLAYER_JOINED {}", player_id);
                let is_reconnecting = room.message_history.iter().any(|msg| msg == &player_joined_msg);

                room.connected_players[player_index] = Some(player.clone());
                tracing::info!(
                    "PLAYER_CONNECTED room_id={} player_id={} reconnecting={}",
                    room_id,
                    player_id,
                    is_reconnecting
                );

                // Always broadcast PLAYER_JOINED (whether new or reconnecting)
                // This tells other players that this player is now online
                let msg = format!("PLAYER_JOINED {}", player_id);

                if !is_reconnecting {
                    // First time connecting - add to history and persist
                    room.message_history.push(msg.clone());

                    let db = self.db.clone();
                    let room_id_str = room_id.to_string();
                    let msg_for_db = msg.clone();
                    tokio::spawn(async move {
                        if let Err(e) = db::append_room_history(&db, &room_id_str, &msg_for_db).await {
                            tracing::error!("Failed to persist PLAYER_JOINED event: {}", e);
                        }
                    });
                }

                // Broadcast to all connected players (including the connecting one)
                for p in room.connected_players.iter() {
                    if let Some(p) = p {
                        let _ = p.send_message(&msg).await;
                    }
                }

                Ok(is_reconnecting)
            } else {
                Err("Player WebSocket not found".to_string())
            }
        } else {
            Err("Room not found".to_string())
        }
    }

    /// Handle WebSocket disconnect - add to disconnected_players, transfer host if needed
    pub async fn on_websocket_disconnect(&self, room_id: &str, player_id: &str) -> Option<String> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;
            // player_id is already in "user:id" format
            let player_thing: Thing = match player_id.parse() {
                Ok(thing) => thing,
                Err(_) => {
                    tracing::error!("Invalid player_id format: {}", player_id);
                    return None;
                }
            };

            // Find player index and mark as offline
            if let Some(player_index) = room.players.iter().position(|p| p == &player_thing) {
                room.connected_players[player_index] = None;

                tracing::info!(
                    "PLAYER_DISCONNECTED room_id={} player={}",
                    room_id,
                    player_thing,
                );

                // Update database to reflect disconnection
                let db = self.db.clone();
                let room_id_str = room_id.to_string();
                let message_history = room.message_history.clone();
                tokio::spawn(async move {
                    if let Err(e) = db::persist_room_state(&db, &room_id_str, message_history).await
                    {
                        tracing::error!("Failed to update room disconnection state: {}", e);
                    }
                });
            }

            // Add PLAYER_LEFT to history (disconnected, not explicit leave)
            let msg = format!("PLAYER_LEFT {}", player_id);
            room.message_history.push(msg.clone());

            // Persist disconnected state to database (async, fire and forget)
            let db = self.db.clone();
            let room_id_str = room_id.to_string();
            let message_history = room.message_history.clone();
            tokio::spawn(async move {
                if let Err(e) = db::persist_room_state(&db, &room_id_str, message_history).await {
                    tracing::error!("Failed to persist disconnected player state: {}", e);
                }
            });

            // Broadcast to remaining connected players
            for player_opt in &room.connected_players {
                if let Some(player) = player_opt {
                    let _ = player.send_message(&msg).await;
                }
            }

            // Handle host disconnect - transfer to next connected player
            if room.host_id == player_id {
                // Find first connected player to be new host
                if let Some(new_host) = room
                    .connected_players
                    .iter()
                    .filter_map(|p| p.as_ref())
                    .next()
                {
                    let new_host_id = new_host.player_id().to_string();
                    room.host_id = new_host_id.clone();
                    // Add to history
                    let host_msg = format!("HOST_CHANGED {}", new_host_id);
                    room.message_history.push(host_msg.clone());
                    for p in room.connected_players.iter() {
                        if let Some(p) = p {
                            let _ = p.send_message(&host_msg).await;
                        }
                    }
                    return Some(new_host_id);
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
                room.message_history.push(message.to_string());

                // Persist event to database (async, fire and forget)
                let db = self.db.clone();
                let room_id = room_id.to_string();
                let message = message.to_string();
                tokio::spawn(async move {
                    if let Err(e) = db::append_room_history(&db, &room_id, &message).await {
                        tracing::error!("Failed to persist room event: {}", e);
                    }
                });
            }

            for player_opt in &room.connected_players {
                if let Some(player) = player_opt {
                    let _ = player.send_message(message).await;
                }
            }
        }
    }

    /// Start a game in a room
    pub async fn start_game<G: Game>(
        self: &Arc<Self>,
        room_id: &str,
        host_id: &str,
        game: &G,
    ) -> Result<Vec<crate::games::GameResult>, String> {
        tracing::info!(
            "RoomManager: Starting game in room {} by host {}",
            room_id,
            host_id
        );

        // Collect players and setup - release lock before running game
        let (players, human_timeout, match_id, game_id) = {
            let rooms = self.rooms.read().await;
            let room_arc = rooms.get(room_id).ok_or("Room not found")?;
            let mut room = room_arc.write().await;

            if room.host_id != host_id {
                tracing::warn!(
                    "RoomManager: Non-host {} tried to start game in room {}",
                    host_id,
                    room_id
                );
                return Err("Only host can start the game".to_string());
            }

            if room.status != "waiting" {
                tracing::warn!(
                    "RoomManager: Game already started or finished in room {} (status: {})",
                    room_id,
                    room.status
                );
                return Err("Game already started or finished".to_string());
            }

            let online_count = room
                .connected_players
                .iter()
                .filter(|p| p.is_some())
                .count();
            if online_count < 2 {
                tracing::warn!(
                    "RoomManager: Not enough online players in room {} ({} online players)",
                    room_id,
                    online_count
                );
                return Err("Need at least 2 connected players to start".to_string());
            }

            tracing::info!(
                "RoomManager: Room {} has {} connected players, starting game",
                room_id,
                room.players.len()
            );
            room.status = "playing".to_string();

            // Sync status change with database
            let mut updates = HashMap::new();
            updates.insert(
                "status".to_string(),
                serde_json::Value::String("playing".to_string()),
            );
            if let Err(e) = db::update_room(&self.db, room_id, updates).await {
                tracing::error!("Failed to sync room status to database: {}", e);
            }
            // Create match record for this game
            let match_id =
                db::create_match(&self.db, room_id, &room.game_id, room.players.clone()).await?;
            // Add GAME_STARTED to history
            room.message_history.push("GAME_STARTED".to_string());

            // Broadcast to all players
            tracing::debug!(
                "RoomManager: Broadcasting GAME_STARTED to {} players",
                room.players.len()
            );
            for (i, p) in room.connected_players.iter().enumerate() {
                if let Some(p) = p {
                    tracing::debug!(
                        "RoomManager: Sending GAME_STARTED to player {} ({})",
                        i,
                        p.player_id()
                    );
                    let _ = p.send_message("GAME_STARTED").await;
                }
            }

            // Get timeout
            let game_metadata = crate::games::find_game_by_id(&room.game_id)
                .ok_or_else(|| format!("Game metadata not found for {}", room.game_id))?;

            let human_timeout = room
                .human_timeout_ms
                .unwrap_or(game_metadata.human_turn_timeout_ms);

            tracing::info!(
                "RoomManager: Using timeout {}ms for room {}",
                human_timeout,
                room_id
            );

            let players: Vec<Box<dyn Player>> = room
                .connected_players
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    if let Some(p) = p {
                        tracing::debug!(
                            "RoomManager: Game will use player {} with ID: {}",
                            i,
                            p.player_id()
                        );
                        Some(Box::new(Arc::clone(p)) as Box<dyn Player>)
                    } else {
                        None
                    }
                })
                .collect();

            (players, human_timeout, match_id, room.game_id.clone())
        }; // Room write lock is released here!

        // Run game without holding room lock - this allows WebSocket handlers to continue processing messages
        tracing::info!(
            "RoomManager: Starting game execution with {} players",
            players.len()
        );

        // Create GameContext for the game to write events
        let game_context = GameContext::new(match_id.clone(), self.db.clone());

        // Write initial game event
        let event = format!("GAME_START game_id={} players={}", game_id, players.len());
        if let Err(e) = self.append_game_event(match_id.clone(), &event).await {
            tracing::error!("Failed to write game start event: {}", e);
        }

        // Run game with timeout to prevent infinite hangs
        let game_timeout = tokio::time::Duration::from_secs(3600); // 1 hour max
        let results = match tokio::time::timeout(game_timeout, game.run(players, human_timeout, game_context)).await {
            Ok(results) => {
                tracing::info!(
                    "RoomManager: Game execution completed with results: {:?}",
                    results
                );
                results
            }
            Err(_) => {
                tracing::error!("RoomManager: Game execution timed out after {:?}", game_timeout);
                // Return timeout results for all players
                let timeout_results = (0..2).map(|_| crate::games::GameResult::TimeLimitExceeded).collect();
                timeout_results
            }
        };

        // Write game completion event
        let results_json = serde_json::to_string(&results).unwrap_or_default();
        let event = format!("GAME_END results={}", results_json);
        if let Err(e) = self.append_game_event(match_id.clone(), &event).await {
            tracing::error!("Failed to write game end event: {}", e);
        }

        // Re-acquire lock to update status and broadcast results
        {
            let rooms = self.rooms.read().await;
            if let Some(room_arc) = rooms.get(room_id) {
                let mut room = room_arc.write().await;

                // Check if game timed out
                let timed_out = results.iter().all(|r| matches!(r, crate::games::GameResult::TimeLimitExceeded));
                let final_status = if timed_out { "crashed" } else { "finished" };
                room.status = final_status.to_string();

                // Sync status change with database
                let mut updates = HashMap::new();
                updates.insert(
                    "status".to_string(),
                    serde_json::Value::String(final_status.to_string()),
                );
                if let Err(e) = db::update_room(&self.db, room_id, updates).await {
                    tracing::error!("Failed to sync room status to database: {}", e);
                }

                // Add GAME_FINISHED to history
                let results_json = serde_json::to_string(&results).unwrap_or_default();
                room.message_history
                    .push(format!("GAME_FINISHED {}", results_json));

                // Broadcast game finished
                tracing::debug!(
                    "RoomManager: Broadcasting GAME_FINISHED to {} players",
                    room.players.len()
                );
                for (i, p) in room.connected_players.iter().enumerate() {
                    if let Some(p) = p {
                        tracing::debug!(
                            "RoomManager: Sending GAME_FINISHED to player {} ({})",
                            i,
                            p.player_id()
                        );
                        let _ = p
                            .send_message(&format!("GAME_FINISHED {}", results_json))
                            .await;
                    }
                }

                tracing::info!(
                    "RoomManager: Game in room {} completed successfully",
                    room_id
                );
            }
        }

        // Complete the match record
        if let Err(e) = db::complete_match(&self.db, match_id.clone()).await {
            tracing::error!("Failed to complete match: {}", e);
        }

        Ok(results)
    }

    /// Remove player from pending
    pub async fn remove_pending_player(&self, player_id: &str) {
        let mut pending = self.pending_players.write().await;
        pending.remove(player_id);
    }


    /// Append game event to match history
    pub async fn append_game_event(&self, match_id: Thing, event: &str) -> Result<(), String> {
        db::append_game_event(&self.db, match_id, event).await
    }

    /// Restore room from database record (for crash recovery)
    async fn restore_room_from_record(&self, record: RoomRecord) -> Result<Room, String> {
        // Parse event_history from persisted entries (JSON format)
        let message_history: Vec<String> = record
            .event_history
            .iter()
            .filter_map(|entry| {
                // Parse JSON format: {"timestamp": ..., "message": "..."}
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(entry) {
                    json.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())
                } else {
                    tracing::warn!("Failed to parse room history entry as JSON: {}", entry);
                    None
                }
            })
            .collect();

        // Get host username - will be rebuilt as players reconnect
        let host_id = record.host_id.id.to_string();
        let room_id = record
            .id
            .as_ref()
            .map(|t| t.id.to_string())
            .ok_or("Room has no ID")?;

        tracing::info!(
            "ROOM_RESTORE room_id={} status={} players={}",
            room_id,
            record.status,
            record.players.len(),
        );

        Ok(Room {
            id: room_id,
            name: record.name,
            game_id: record.game_id,
            host_id,
            players: record.players.clone(),
            max_players: record.max_players as usize,
            connected_players: Vec::new(),
            status: record.status,
            human_timeout_ms: record.human_timeout_ms,
            message_history,
        })
    }

    /// Wait for another task to finish restoring a room
    async fn wait_for_restoration(&self, room_id: &str) -> Result<(), String> {
        let max_wait = tokio::time::Duration::from_secs(10);
        let start = tokio::time::Instant::now();

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let restoring = self.restoring_rooms.read().await;
            if !restoring.contains(room_id) {
                return Ok(());
            }

            if start.elapsed() > max_wait {
                return Err("Timeout waiting for room restoration".to_string());
            }
        }
    }

    /// Get websocket players for a room
    pub async fn get_connected_players(&self, room_id: &str) -> Vec<Option<Arc<crate::models::players::HumanPlayer>>> {
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let room = room_arc.read().await;
            return room.connected_players.clone();
        }
        Vec::new()
    }

    /// Recover orphaned rooms on server startup
    /// Marks "playing" rooms from previous crashed server instances as "crashed"
    pub async fn recover_orphaned_rooms(&self) -> Result<(), String> {
        tracing::info!("Starting orphaned room recovery...");

        // Find all rooms with "playing" status
        let playing_rooms = db::list_rooms(&self.db, None, Some("playing")).await
            .map_err(|e| format!("Failed to list playing rooms: {}", e))?;

        for room_record in playing_rooms {
            let room_id = room_record
                .id
                .as_ref()
                .map(|t| t.to_string())
                .ok_or("Room has no ID")?;

            tracing::warn!("Found orphaned room {} in 'playing' status, marking as 'crashed'", room_id);

            // Mark room as crashed so players know to restart
            let mut updates = HashMap::new();
            updates.insert(
                "status".to_string(),
                serde_json::Value::String("crashed".to_string()),
            );

            if let Err(e) = db::update_room(&self.db, &room_id, updates).await {
                tracing::error!("Failed to update orphaned room {}: {}", room_id, e);
                continue;
            }

            tracing::info!("Successfully marked room {} as crashed", room_id);
        }

        tracing::info!("Orphaned room recovery complete");
        Ok(())
    }
}
