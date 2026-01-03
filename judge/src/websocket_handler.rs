use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures_util::stream::StreamExt;
use std::sync::Arc;

use crate::game_logic::GameLogic;
use crate::players::HumanPlayer;
use crate::player::Player;
use crate::app_state::AppState;

pub async fn websocket_room_handler<G: GameLogic + Clone + Send + Sync + 'static>(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState<G>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, room_id, state))
}

async fn handle_websocket<G: GameLogic + Clone + Send + Sync + 'static>(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState<G>>,
) {
    let (sender, mut receiver) = socket.split();
    let (move_tx, move_rx) = tokio::sync::mpsc::unbounded_channel();
    
    let player_id = format!("ws-{}", uuid::Uuid::new_v4());
    let player = Arc::new(HumanPlayer::new(player_id.clone(), sender, move_rx));
    
    // Add WebSocket player to pending players
    state.room_manager.add_websocket_player(&player_id, player.clone()).await;
    
    // Try to connect player to room (they should have joined via HTTP first)
    match state.room_manager.connect_player_to_room(&room_id, &player_id).await {
        Ok(()) => {
            let welcome_msg = format!("CONNECTED {}", room_id);
            
            if let Err(e) = player.send_message(&welcome_msg).await {
                tracing::error!("Failed to send welcome message: {}", e);
                return;
            }
        }
        Err(e) => {
            let error_msg = format!("ERROR {}", e);
            let _ = player.send_message(&error_msg).await;
            return;
        }
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text = text.trim();
                
                if text == "EXIT" {
                    // Leave room gracefully
                    state.room_manager.leave_room(&room_id, &player_id).await;
                    let leave_msg = "LEFT_ROOM";
                    let _ = player.send_message(leave_msg).await;
                    break;
                } else if text.starts_with("CHAT ") {
                    // Chat message - broadcast to other players in room
                    let chat_content = &text[5..]; // Remove "CHAT " prefix
                    let _chat_msg = format!("CHAT {} {}", player_id, chat_content);
                    // TODO: Broadcast to other players in room
                    tracing::info!("Chat from {}: {}", player_id, chat_content);
                } else if text == "START" {
                    // Host is starting the game
                    match state.room_manager.start_game(&room_id, &player_id, &state.game).await {
                        Ok(results) => {
                            let result_msg = format!("GAME_FINISHED {:?}", results);
                            let _ = player.send_message(&result_msg).await;
                        }
                        Err(e) => {
                            let error_msg = format!("ERROR {}", e);
                            let _ = player.send_message(&error_msg).await;
                        }
                    }
                } else {
                    // Forward as game move
                    if let Err(e) = move_tx.send(text.to_string()) {
                        tracing::error!("Failed to forward move: {}", e);
                        break;
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
            _ => {}
        }
    }

    // Cleanup on disconnect
    state.room_manager.leave_room(&room_id, &player_id).await;
    tracing::info!("Player {} left room {}", player_id, room_id);
}
