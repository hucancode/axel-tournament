use crate::app_state::AppState;
use crate::middleware::auth::validate_jwt;
use crate::models::game::Game;
use crate::models::players::{HumanPlayer, Player};
use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use surrealdb::sql::Thing;

/// WebSocket upgrade handler
pub async fn ws_get_room<G: Game + Clone + Send + Sync + 'static>(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState<G>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, room_id, state))
}

/// Main WebSocket connection handler
async fn handle_websocket<G: Game + Clone + Send + Sync + 'static>(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState<G>>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for first message which must be "LOGIN <jwt>" (with 10 second timeout)
    let login_timeout = tokio::time::Duration::from_secs(10);
    let player_id = match tokio::time::timeout(login_timeout, receiver.next()).await {
        Err(_) => {
            let _ = sender
                .send(Message::Text("LOGIN_FAILED Login timeout".into()))
                .await;
            return;
        }
        Ok(msg_result) => match msg_result {
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
                    Ok(claims) => {
                        // Validation will be done in connect_player_to_room after loading room from DB if needed
                        claims.sub
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
        }
    };

    // Now create the player with validated player_id (already in "user:id" format)
    let (move_tx, move_rx) = tokio::sync::mpsc::unbounded_channel();
    let player_thing: Thing = match player_id.parse() {
        Ok(thing) => thing,
        Err(_) => {
            let _ = sender
                .send(Message::Text("LOGIN_FAILED Invalid player_id format".into()))
                .await;
            return;
        }
    };
    let player = Arc::new(HumanPlayer::new(
        player_thing,
        sender,
        move_rx,
    ));

    tracing::debug!(
        "WebSocket {}: Created new HumanPlayer instance with fresh channels",
        player_id
    );

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
        Ok(is_reconnecting) => {
            if is_reconnecting {
                // Send LOGIN_OK with RECONNECT flag
                let _ = player
                    .send_message(&format!("LOGIN_OK {} RECONNECT", player_id))
                    .await;
                let _ = player.send_message("REPLAY_START").await;

                // Fetch both room and match history atomically to ensure consistency
                match crate::services::room::db::get_room_and_match_history_atomic(&state.db, &room_id).await {
                    Ok((room_history, match_history)) => {
                        // Send room-level message history first (before game events)
                        for msg in room_history {
                            let _ = player.send_message(&msg).await;
                        }

                        // Send game-level reconnection state
                        // First restore game state from match history
                        let game = state.game.clone();
                        if !match_history.is_empty() {
                            game.restore_from_events(&match_history);
                        }

                        // Send reconnection state to client
                        let reconnect_messages = game.get_event_source(&player_id);
                        for msg in reconnect_messages {
                            let _ = player.send_message(&msg).await;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch reconnection history: {}", e);
                        let _ = player.send_message(&format!("ERROR Failed to load game state: {}", e)).await;
                    }
                }

                let _ = player.send_message("REPLAY_END").await;
            } else {
                // Send LOGIN_OK for new connection FIRST
                let _ = player
                    .send_message(&format!("LOGIN_OK {}", player_id))
                    .await;

                // Then send info about existing players
                let connected_players = state.room_manager.get_connected_players(&room_id).await;
                for p in connected_players.iter() {
                    if let Some(p) = p {
                        let msg = format!("PLAYER_JOINED {}", p.player_id());
                        let _ = p.send_message(&msg).await;
                    }
                }
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
                                tracing::info!(
                                    "WebSocket {}: Game started successfully",
                                    player_id_clone
                                );
                                // Game results are handled by the game logic itself
                            }
                            Err(e) => {
                                tracing::error!(
                                    "WebSocket {}: Failed to start game: {}",
                                    player_id_clone,
                                    e
                                );
                                let error_msg = format!("ERROR {}", e);
                                let _ = player_clone.send_message(&error_msg).await;
                            }
                        }
                    });
                } else {
                    // Forward as game move
                    tracing::info!(
                        "WebSocket {}: Received game move from client: '{}'",
                        player_id,
                        text
                    );
                    match move_tx.send(text.to_string()) {
                        Ok(_) => {
                            tracing::info!(
                                "WebSocket {}: Successfully forwarded move to game channel",
                                player_id
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "WebSocket {}: Failed to forward move to channel: {}",
                                player_id,
                                e
                            );
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
