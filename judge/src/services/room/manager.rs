// RoomManager - Core orchestration for room lifecycle and game execution

use super::db;
use crate::db::Database;
use crate::models::game::Game;
use crate::models::players::{HumanPlayer, Player};
use crate::models::room::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use surrealdb::sql::Thing;
use tokio::sync::RwLock;

pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<Room>>>>>,
    pending_players: Arc<RwLock<HashMap<String, Arc<HumanPlayer>>>>,
    db: Database,
    restoring_rooms: Arc<RwLock<HashSet<String>>>,
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

    // ========================================================================
    // Room Loading
    // ========================================================================

    /// Ensure room is loaded into memory, loading from DB if needed
    async fn ensure_room_loaded(&self, room_id: &str) -> Result<(), String> {
        // Fast path: already in memory
        if self.rooms.read().await.contains_key(room_id) {
            return Ok(());
        }

        // Check if another task is restoring this room
        if self.restoring_rooms.read().await.contains(room_id) {
            return self.wait_for_restoration(room_id).await;
        }

        // Mark as restoring
        self.restoring_rooms.write().await.insert(room_id.to_string());

        let result = async {
            let record = db::get_room(&self.db, room_id).await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or("Room not found")?;

            let room = self.restore_room_from_record(record)?;
            self.rooms.write().await.insert(room_id.to_string(), Arc::new(RwLock::new(room)));
            Ok(())
        }.await;

        self.restoring_rooms.write().await.remove(room_id);
        result
    }

    async fn wait_for_restoration(&self, room_id: &str) -> Result<(), String> {
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(10);
        while tokio::time::Instant::now() < deadline {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if !self.restoring_rooms.read().await.contains(room_id) {
                return if self.rooms.read().await.contains_key(room_id) {
                    Ok(())
                } else {
                    Err("Room restoration failed".to_string())
                };
            }
        }
        Err("Timeout waiting for room restoration".to_string())
    }

    fn restore_room_from_record(&self, record: RoomRecord) -> Result<Room, String> {
        let message_history: Vec<String> = record.event_history.iter()
            .filter_map(|entry| {
                serde_json::from_str::<serde_json::Value>(entry).ok()
                    .and_then(|json| json.get("message")?.as_str().map(String::from))
            })
            .collect();

        let room_id = record.id.as_ref().map(|t| t.to_string()).ok_or("Room has no ID")?;

        Ok(Room {
            id: room_id,
            name: record.name,
            game_id: record.game_id,
            host_id: record.host_id.to_string(),
            players: record.players.clone(),
            max_players: record.max_players as usize,
            connected_players: vec![None; record.players.len()],
            status: record.status,
            human_timeout_ms: record.human_timeout_ms,
            message_history,
        })
    }

    // ========================================================================
    // Room CRUD
    // ========================================================================

    pub async fn create_room(
        &self,
        name: String,
        game_id: String,
        host_id: String,
        max_players: usize,
        human_timeout_ms: Option<u64>,
    ) -> Result<Room, String> {
        let host_thing: Thing = host_id.parse().map_err(|_| "Invalid host_id format")?;

        let room_record = db::create_room(
            &self.db, game_id.clone(), host_thing.clone(), name.clone(),
            max_players as u32, human_timeout_ms,
        ).await.map_err(|e| format!("Failed to create room: {}", e))?;

        let room_id = room_record.id.as_ref()
            .map(|t| t.to_string())
            .ok_or("Database did not return room ID")?;

        let room = Room {
            id: room_id.clone(),
            name,
            game_id,
            host_id: host_id.clone(),
            players: vec![host_thing],
            connected_players: vec![None],
            max_players,
            status: "waiting".to_string(),
            human_timeout_ms,
            message_history: Vec::new(),
        };

        self.rooms.write().await.insert(room_id, Arc::new(RwLock::new(room.clone())));
        Ok(room)
    }

    pub async fn get_room(&self, room_id: &str) -> Option<RoomResponse> {
        // Try memory first
        if let Some(room_arc) = self.rooms.read().await.get(room_id) {
            return Some(room_arc.read().await.to_response());
        }

        // Fall back to database
        let record = db::get_room(&self.db, room_id).await.ok()??;
        let players: Vec<PlayerInfo> = record.players.iter()
            .map(|t| PlayerInfo {
                id: t.to_string(),
                username: "Unknown".to_string(),
                connected: false,
            })
            .collect();

        Some(RoomResponse {
            id: record.id.as_ref().map(|t| t.to_string()).unwrap_or_default(),
            name: record.name,
            game_id: record.game_id,
            max_players: record.max_players,
            status: record.status,
            host_id: record.host_id.to_string(),
            players,
            reconnecting: false,
        })
    }

    pub async fn list_rooms(&self, game_id: Option<&str>) -> Vec<RoomListItem> {
        let Ok(records) = db::list_rooms(&self.db, game_id, Some("waiting")).await else {
            return Vec::new();
        };

        let rooms = self.rooms.read().await;
        let mut result = Vec::with_capacity(records.len());

        for record in records {
            let room_id = record.id.as_ref().map(|t| t.to_string()).unwrap_or_default();
            let (current_players, status) = match rooms.get(&room_id) {
                Some(r) => {
                    let room = r.read().await;
                    (room.connected_count(), room.status.clone())
                }
                None => (0, record.status.clone()),
            };

            result.push(RoomListItem {
                id: room_id,
                name: record.name,
                game_id: record.game_id,
                host_username: "Unknown".to_string(),
                current_players,
                max_players: record.max_players as usize,
                status,
            });
        }
        result
    }

    async fn delete_room(&self, room_id: &str) {
        if self.rooms.write().await.remove(room_id).is_some() {
            tracing::info!("Room {} deleted", room_id);
            let _ = db::delete_room(&self.db, room_id).await;
        }
    }

    // ========================================================================
    // Player Connection (merged join + connect)
    // ========================================================================

    pub async fn add_websocket_player(&self, player_id: &str, player: Arc<HumanPlayer>) {
        self.pending_players.write().await.insert(player_id.to_string(), player);
    }

    pub async fn remove_pending_player(&self, player_id: &str) {
        self.pending_players.write().await.remove(player_id);
    }

    /// Join room and connect WebSocket in one step. Returns is_reconnecting.
    pub async fn join_and_connect(&self, room_id: &str, player_id: &str) -> Result<bool, String> {
        self.ensure_room_loaded(room_id).await?;

        let player_thing: Thing = player_id.parse().map_err(|_| "Invalid player_id")?;

        let rooms = self.rooms.read().await;
        let room_arc = rooms.get(room_id).ok_or("Room not found")?;
        let mut room = room_arc.write().await;

        // Check if player already in room (reconnection) or needs to join
        let (player_index, is_reconnecting) = match room.players.iter().position(|p| p == &player_thing) {
            Some(idx) => {
                let was_connected = room.connected_players.get(idx).and_then(|p| p.as_ref()).is_some();
                (idx, !was_connected)
            }
            None => {
                // New player - validate and add
                if room.status != "waiting" {
                    return Err("Room is not accepting new players".to_string());
                }
                if room.connected_count() >= room.max_players {
                    return Err("Room is full".to_string());
                }

                room.players.push(player_thing.clone());
                room.connected_players.push(None);

                // Persist to DB
                let db = self.db.clone();
                let rid = room_id.to_string();
                let pt = player_thing.clone();
                tokio::spawn(async move { let _ = db::add_player(&db, &rid, pt).await; });

                (room.players.len() - 1, false)
            }
        };

        // Link WebSocket
        let pending = self.pending_players.read().await;
        let player = pending.get(player_id).ok_or("WebSocket not found")?.clone();
        room.connected_players[player_index] = Some(player);

        // Broadcast and persist PLAYER_JOINED
        let msg = format!("PLAYER_JOINED {}", player_id);
        let already_joined = room.message_history.iter().any(|m| m == &msg);

        if !already_joined {
            room.message_history.push(msg.clone());
            let db = self.db.clone();
            let rid = room_id.to_string();
            let m = msg.clone();
            tokio::spawn(async move { let _ = db::append_room_history(&db, &rid, &m).await; });
        }

        room.broadcast(&msg).await;
        Ok(is_reconnecting || already_joined)
    }

    // ========================================================================
    // Player Disconnect / Leave
    // ========================================================================

    pub async fn leave_room(&self, room_id: &str, player_id: &str) -> LeaveResult {
        let rooms = self.rooms.read().await;
        let Some(room_arc) = rooms.get(room_id) else {
            return LeaveResult::NotInRoom;
        };

        let mut room = room_arc.write().await;
        let Ok(player_thing) = player_id.parse::<Thing>() else {
            return LeaveResult::NotInRoom;
        };

        let Some(idx) = room.players.iter().position(|p| p == &player_thing) else {
            return LeaveResult::NotInRoom;
        };

        // Remove player
        room.players.remove(idx);
        room.connected_players.remove(idx);

        // Persist
        let db = self.db.clone();
        let rid = room_id.to_string();
        let pt = player_thing.clone();
        tokio::spawn(async move { let _ = db::remove_player(&db, &rid, pt).await; });

        // Broadcast leave
        let msg = format!("PLAYER_LEFT {}", player_id);
        room.message_history.push(msg.clone());
        room.broadcast(&msg).await;

        // Handle host transfer or room deletion
        if room.host_id == player_id {
            if room.players.is_empty() {
                drop(room);
                drop(rooms);
                self.delete_room(room_id).await;
                return LeaveResult::RoomDeleted;
            }

            if room.transfer_host_if_needed(player_id).await.is_some() {
                self.persist_room_state(room_id, &room).await;
                return LeaveResult::HostTransferred;
            }
        }

        LeaveResult::Left
    }

    pub async fn on_websocket_disconnect(&self, room_id: &str, player_id: &str) {
        let rooms = self.rooms.read().await;
        let Some(room_arc) = rooms.get(room_id) else { return };

        let mut room = room_arc.write().await;
        let Ok(player_thing) = player_id.parse::<Thing>() else { return };

        // Mark player as disconnected
        if let Some(idx) = room.players.iter().position(|p| p == &player_thing) {
            room.connected_players[idx] = None;
        }

        // Broadcast and persist
        let msg = format!("PLAYER_LEFT {}", player_id);
        room.message_history.push(msg.clone());
        room.broadcast(&msg).await;

        room.transfer_host_if_needed(player_id).await;
        self.persist_room_state(room_id, &room).await;
    }

    pub async fn broadcast_to_room(&self, room_id: &str, message: &str, add_to_history: bool) {
        let rooms = self.rooms.read().await;
        let Some(room_arc) = rooms.get(room_id) else { return };

        let mut room = room_arc.write().await;
        if add_to_history {
            room.message_history.push(message.to_string());
            let db = self.db.clone();
            let rid = room_id.to_string();
            let m = message.to_string();
            tokio::spawn(async move { let _ = db::append_room_history(&db, &rid, &m).await; });
        }
        room.broadcast(message).await;
    }

    async fn persist_room_state(&self, room_id: &str, room: &Room) {
        let db = self.db.clone();
        let rid = room_id.to_string();
        let history = room.message_history.clone();
        tokio::spawn(async move { let _ = db::persist_room_state(&db, &rid, history).await; });
    }

    pub async fn get_connected_players(&self, room_id: &str) -> Vec<Option<Arc<HumanPlayer>>> {
        let rooms = self.rooms.read().await;
        match rooms.get(room_id) {
            Some(r) => r.read().await.connected_players.clone(),
            None => Vec::new(),
        }
    }

    // ========================================================================
    // Game Execution
    // ========================================================================

    pub async fn start_game<G: Game>(
        self: &Arc<Self>,
        room_id: &str,
        host_id: &str,
        game: &G,
    ) -> Result<Vec<crate::games::GameResult>, String> {
        let ctx = self.prepare_game(room_id, host_id).await?;
        let match_id = ctx.match_id.clone();
        let results = self.run_game(game, ctx).await;
        self.finalize_game(room_id, &match_id, &results).await;
        Ok(results)
    }

    async fn prepare_game(&self, room_id: &str, host_id: &str) -> Result<GameStartContext, String> {
        let rooms = self.rooms.read().await;
        let room_arc = rooms.get(room_id).ok_or("Room not found")?;
        let mut room = room_arc.write().await;

        if room.host_id != host_id {
            return Err("Only host can start the game".to_string());
        }
        if room.status != "waiting" {
            return Err("Game already started or finished".to_string());
        }
        if room.connected_count() < 2 {
            return Err("Need at least 2 connected players to start".to_string());
        }

        room.status = "playing".to_string();

        // Sync status to DB
        let mut updates = HashMap::new();
        updates.insert("status".to_string(), serde_json::Value::String("playing".to_string()));
        let _ = db::update_room(&self.db, room_id, updates).await;

        // Create match record
        let match_id = db::create_match(&self.db, room_id, &room.game_id, room.players.clone()).await?;

        // Broadcast game started
        room.message_history.push("GAME_STARTED".to_string());
        room.broadcast("GAME_STARTED").await;

        // Get timeout
        let game_metadata = crate::games::find_game_by_id(&room.game_id)
            .ok_or_else(|| format!("Game metadata not found for {}", room.game_id))?;
        let timeout_ms = room.human_timeout_ms.unwrap_or(game_metadata.human_turn_timeout_ms);

        // Collect players
        let players: Vec<Box<dyn Player>> = room.connected_players.iter()
            .flatten()
            .map(|p| Box::new(Arc::clone(p)) as Box<dyn Player>)
            .collect();

        Ok(GameStartContext {
            match_id,
            game_id: room.game_id.clone(),
            players,
            timeout_ms,
        })
    }

    async fn run_game<G: Game>(&self, game: &G, ctx: GameStartContext) -> Vec<crate::games::GameResult> {
        let game_context = GameContext::new(ctx.match_id.clone(), self.db.clone());

        // Write start event
        let _ = db::append_game_event(
            &self.db,
            ctx.match_id.clone(),
            &format!("GAME_START game_id={} players={}", ctx.game_id, ctx.players.len()),
        ).await;

        // Run with 1 hour timeout
        let timeout = tokio::time::Duration::from_secs(3600);
        match tokio::time::timeout(timeout, game.run(ctx.players, ctx.timeout_ms, game_context)).await {
            Ok(results) => results,
            Err(_) => {
                tracing::error!("Game timed out");
                vec![crate::games::GameResult::TimeLimitExceeded; 2]
            }
        }
    }

    async fn finalize_game(&self, room_id: &str, match_id: &Thing, results: &[crate::games::GameResult]) {
        // Write end event
        let results_json = serde_json::to_string(results).unwrap_or_default();
        let _ = db::append_game_event(&self.db, match_id.clone(), &format!("GAME_END results={}", results_json)).await;

        // Update room status
        let rooms = self.rooms.read().await;
        if let Some(room_arc) = rooms.get(room_id) {
            let mut room = room_arc.write().await;

            let timed_out = results.iter().all(|r| matches!(r, crate::games::GameResult::TimeLimitExceeded));
            room.status = if timed_out { "crashed" } else { "finished" }.to_string();

            let mut updates = HashMap::new();
            updates.insert("status".to_string(), serde_json::Value::String(room.status.clone()));
            let _ = db::update_room(&self.db, room_id, updates).await;

            let msg = format!("GAME_FINISHED {}", results_json);
            room.message_history.push(msg.clone());
            room.broadcast(&msg).await;
        }

        let _ = db::complete_match(&self.db, match_id.clone()).await;
    }

    pub async fn append_game_event(&self, match_id: Thing, event: &str) -> Result<(), String> {
        db::append_game_event(&self.db, match_id, event).await
    }

    // ========================================================================
    // Recovery
    // ========================================================================

    pub async fn recover_orphaned_rooms(&self) -> Result<(), String> {
        let playing_rooms = db::list_rooms(&self.db, None, Some("playing")).await
            .map_err(|e| format!("Failed to list playing rooms: {}", e))?;

        for record in playing_rooms {
            let room_id = record.id.as_ref().map(|t| t.to_string()).ok_or("Room has no ID")?;
            tracing::warn!("Marking orphaned room {} as crashed", room_id);

            let mut updates = HashMap::new();
            updates.insert("status".to_string(), serde_json::Value::String("crashed".to_string()));
            let _ = db::update_room(&self.db, &room_id, updates).await;
        }

        Ok(())
    }
}

struct GameStartContext {
    match_id: Thing,
    game_id: String,
    players: Vec<Box<dyn Player>>,
    timeout_ms: u64,
}
