use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc},
};
use tokio::{
    sync::{broadcast, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, RoomSession>>>,
}

#[derive(Clone)]
pub struct RoomSession {
    pub room_id: String,
    pub game_id: String,
    pub players: HashMap<String, PlayerConnection>,
    pub tx: broadcast::Sender<GameMessage>,
    pub status: RoomStatus,
}

#[derive(Clone)]
pub struct PlayerConnection {
    pub user_id: String,
    pub username: String,
    pub player_number: u8,
}

#[derive(Clone, Debug)]
pub enum RoomStatus {
    Waiting,
    Playing,
    Finished,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameMessage {
    #[serde(rename = "player_joined")]
    PlayerJoined { user_id: String, username: String },
    #[serde(rename = "player_left")]
    PlayerLeft { user_id: String },
    #[serde(rename = "game_started")]
    GameStarted,
    #[serde(rename = "game_move")]
    GameMove { user_id: String, data: String },
    #[serde(rename = "game_state")]
    GameState { data: String },
    #[serde(rename = "chat_message")]
    ChatMessage { user_id: String, username: String, message: String },
    #[serde(rename = "error")]
    Error { message: String },
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let user_id = Uuid::new_v4().to_string();
    let username = format!("Player_{}", &user_id[..8]);

    let mut rx = {
        let mut rooms = state.rooms.lock().await;
        let room = rooms.entry(room_id.clone()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            RoomSession {
                room_id: room_id.clone(),
                game_id: "game_123".to_string(),
                players: HashMap::new(),
                tx,
                status: RoomStatus::Waiting,
            }
        });

        let player_number = (room.players.len() + 1) as u8;
        room.players.insert(user_id.clone(), PlayerConnection {
            user_id: user_id.clone(),
            username: username.clone(),
            player_number,
        });

        let _ = room.tx.send(GameMessage::PlayerJoined {
            user_id: user_id.clone(),
            username: username.clone(),
        });

        room.tx.subscribe()
    };

    let tx_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let room_id_clone = room_id.clone();
    let user_id_clone = user_id.clone();

    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Ok(parsed) = serde_json::from_str::<IncomingMessage>(&text) {
                handle_incoming_message(parsed, &state_clone, &room_id_clone, &user_id_clone, &username).await;
            }
        } else if msg.is_err() {
            break;
        }
    }

    {
        let mut rooms = state.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.players.remove(&user_id);
            let _ = room.tx.send(GameMessage::PlayerLeft { user_id });

            if room.players.is_empty() {
                rooms.remove(&room_id);
            }
        }
    }

    tx_task.abort();
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum IncomingMessage {
    #[serde(rename = "game_move")]
    GameMove { data: String },
    #[serde(rename = "chat_message")]
    ChatMessage { message: String },
    #[serde(rename = "start_game")]
    StartGame,
}

async fn handle_incoming_message(
    msg: IncomingMessage,
    state: &AppState,
    room_id: &str,
    user_id: &str,
    username: &str,
) {
    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.get_mut(room_id) {
        match msg {
            IncomingMessage::GameMove { data } => {
                let _ = room.tx.send(GameMessage::GameMove {
                    user_id: user_id.to_string(),
                    data,
                });
            }
            IncomingMessage::ChatMessage { message } => {
                let _ = room.tx.send(GameMessage::ChatMessage {
                    user_id: user_id.to_string(),
                    username: username.to_string(),
                    message,
                });
            }
            IncomingMessage::StartGame => {
                if room.players.len() >= 2 && matches!(room.status, RoomStatus::Waiting) {
                    room.status = RoomStatus::Playing;
                    let _ = room.tx.send(GameMessage::GameStarted);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/ws/room/:room_id", get(websocket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("Interactive judge listening on 0.0.0.0:8081");

    axum::serve(listener, app).await.unwrap();
}
