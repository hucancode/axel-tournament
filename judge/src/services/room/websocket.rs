use crate::app_state::AppState;
use crate::middleware::auth::validate_jwt;
use crate::models::game::Game;
use crate::models::players::{HumanPlayer, Player};
use axum::{
    extract::{Path, State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use surrealdb::sql::Thing;

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

    // Authenticate via LOGIN message
    let login_timeout = tokio::time::Duration::from_secs(10);
    let player_id = match tokio::time::timeout(login_timeout, receiver.next()).await {
        Err(_) => {
            let _ = sender.send(Message::Text("LOGIN_FAILED Login timeout".into())).await;
            return;
        }
        Ok(Some(Ok(Message::Text(text)))) => {
            let text = text.trim();
            if !text.starts_with("LOGIN ") {
                let _ = sender.send(Message::Text("LOGIN_FAILED Missing LOGIN command".into())).await;
                return;
            }

            match validate_jwt(&text[6..], &state.jwt_secret) {
                Ok(claims) => claims.sub,
                Err(e) => {
                    let _ = sender.send(Message::Text(format!("LOGIN_FAILED {}", e).into())).await;
                    return;
                }
            }
        }
        _ => {
            let _ = sender.send(Message::Text("LOGIN_FAILED Connection closed".into())).await;
            return;
        }
    };

    // Create player
    let (move_tx, move_rx) = tokio::sync::mpsc::unbounded_channel();
    let Ok(player_thing) = player_id.parse::<Thing>() else {
        let _ = sender.send(Message::Text("LOGIN_FAILED Invalid player_id format".into())).await;
        return;
    };

    let player = Arc::new(HumanPlayer::new(player_thing, sender, move_rx));
    state.room_manager.add_websocket_player(&player_id, player.clone()).await;

    // Join room and connect
    match state.room_manager.join_and_connect(&room_id, &player_id).await {
        Ok(is_reconnecting) => {
            if is_reconnecting {
                let _ = player.send_message(&format!("LOGIN_OK {} RECONNECT", player_id)).await;
                let _ = player.send_message("REPLAY_START").await;

                if let Ok((room_history, match_history)) =
                    crate::services::room::db::get_room_and_match_history_atomic(&state.db, &room_id).await
                {
                    for msg in room_history {
                        let _ = player.send_message(&msg).await;
                    }
                    let game = state.game.clone();
                    if !match_history.is_empty() {
                        game.restore_from_events(&match_history);
                    }
                    for msg in game.get_event_source(&player_id) {
                        let _ = player.send_message(&msg).await;
                    }
                }
                let _ = player.send_message("REPLAY_END").await;
            } else {
                let _ = player.send_message(&format!("LOGIN_OK {}", player_id)).await;
            }
        }
        Err(e) => {
            let _ = player.send_message(&format!("ERROR {}", e)).await;
            return;
        }
    }

    // Message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text = text.trim();

                if text == "LEAVE" {
                    state.room_manager.leave_room(&room_id, &player_id).await;
                    let _ = player.send_message("LEFT_ROOM").await;
                    break;
                } else if text.starts_with("CHAT ") {
                    let chat_msg = format!("CHAT {} {}", player_id, &text[5..]);
                    state.room_manager.broadcast_to_room(&room_id, &chat_msg, true).await;
                } else if text == "START" {
                    let room_id = room_id.clone();
                    let player_id = player_id.clone();
                    let state = state.clone();
                    let player = player.clone();
                    tokio::spawn(async move {
                        if let Err(e) = state.room_manager.start_game(&room_id, &player_id, &state.game).await {
                            let _ = player.send_message(&format!("ERROR {}", e)).await;
                        }
                    });
                } else if move_tx.send(text.to_string()).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    // Cleanup
    state.room_manager.on_websocket_disconnect(&room_id, &player_id).await;
    state.room_manager.remove_pending_player(&player_id).await;
}
