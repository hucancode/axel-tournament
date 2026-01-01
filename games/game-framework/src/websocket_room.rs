use crate::game_trait::GameLogic;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameMessage {
    PlayerJoined {
        player_id: String,
        player_number: u8,
    },
    PlayerLeft {
        player_id: String,
    },
    GameStarted {
        players: Vec<String>,
    },
    GameState {
        state: serde_json::Value,
    },
    RoundResult {
        result: serde_json::Value,
    },
    GameOver {
        result: serde_json::Value,
    },
    Error {
        message: String,
    },
}

struct PlayerState<M> {
    player_id: String,
    player_number: u8,
    current_move: Option<M>,
}

pub struct RoomState<G: GameLogic> {
    pub room_id: String,
    pub players: Arc<RwLock<HashMap<String, PlayerState<G::Move>>>>,
    pub game_state: Arc<RwLock<G::GameState>>,
    pub max_rounds: u32,
    pub current_round: Arc<RwLock<u32>>,
    pub tx: broadcast::Sender<GameMessage>,
}

impl<G: GameLogic> Clone for RoomState<G> {
    fn clone(&self) -> Self {
        RoomState {
            room_id: self.room_id.clone(),
            players: Arc::clone(&self.players),
            game_state: Arc::clone(&self.game_state),
            max_rounds: self.max_rounds,
            current_round: Arc::clone(&self.current_round),
            tx: self.tx.clone(),
        }
    }
}

pub type AppState<G> = Arc<RwLock<HashMap<String, RoomState<G>>>>;

pub async fn websocket_handler<G: GameLogic>(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState<G>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket::<G>(socket, room_id, state))
}

async fn handle_socket<G: GameLogic>(
    socket: WebSocket,
    room_id: String,
    state: AppState<G>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Get or create room
    let room_state = {
        let mut rooms = state.write().await;
        rooms
            .entry(room_id.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                RoomState {
                    room_id: room_id.clone(),
                    players: Arc::new(RwLock::new(HashMap::new())),
                    game_state: Arc::new(RwLock::new(G::new_game())),
                    max_rounds: 100,
                    current_round: Arc::new(RwLock::new(0)),
                    tx,
                }
            })
            .clone()
    };

    // Assign player
    let player_id = format!("player_{}", uuid::Uuid::new_v4());
    let player_number = {
        let mut players = room_state.players.write().await;
        let player_number = match players.len() {
            0 => 1,
            1 => 2,
            _ => {
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&GameMessage::Error {
                            message: "Room is full".to_string(),
                        })
                        .unwrap(),
                    ))
                    .await;
                return;
            }
        };
        players.insert(
            player_id.clone(),
            PlayerState {
                player_id: player_id.clone(),
                player_number,
                current_move: None,
            },
        );
        player_number
    };

    info!(
        "Player {} joined room {} as Player {}",
        player_id, room_id, player_number
    );

    // Notify others
    let _ = room_state.tx.send(GameMessage::PlayerJoined {
        player_id: player_id.clone(),
        player_number,
    });

    // Subscribe to broadcasts
    let mut rx = room_state.tx.subscribe();

    // Start game if 2 players
    {
        let players = room_state.players.read().await;
        if players.len() == 2 {
            let player_list: Vec<String> = players.keys().cloned().collect();
            let _ = room_state.tx.send(GameMessage::GameStarted {
                players: player_list,
            });

            // Send initial game state
            let game_state = room_state.game_state.read().await;
            let state_msg = G::get_state_message(&game_state);
            let _ = room_state.tx.send(GameMessage::GameState { state: state_msg });
        }
    }

    // Handle messages
    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                if let Message::Text(text) = msg {
                    if let Ok(move_req) = serde_json::from_str::<MoveRequest>(&text) {
                        handle_move::<G>(&room_state, &player_id, move_req.player_move).await;
                    }
                }
            }
            Ok(msg) = rx.recv() => {
                let json = serde_json::to_string(&msg).unwrap();
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
            else => break,
        }
    }

    // Player disconnected
    {
        let mut players = room_state.players.write().await;
        players.remove(&player_id);
    }
    let _ = room_state.tx.send(GameMessage::PlayerLeft {
        player_id: player_id.clone(),
    });

    info!("Player {} left room {}", player_id, room_id);
}

async fn handle_move<G: GameLogic>(
    room: &RoomState<G>,
    player_id: &str,
    move_str: String,
) {
    // Parse move
    let player_move = match G::parse_move(&move_str) {
        Ok(m) => m,
        Err(_) => {
            let _ = room.tx.send(GameMessage::Error {
                message: "Invalid move".to_string(),
            });
            return;
        }
    };

    // Record player's move
    let player_idx = {
        let mut players = room.players.write().await;
        if let Some(player) = players.get_mut(player_id) {
            player.current_move = Some(player_move.clone());
            player.player_number as usize - 1
        } else {
            return;
        }
    };

    // Check if both players have moved
    let both_moved = {
        let players = room.players.read().await;
        players.len() == 2 && players.values().all(|p| p.current_move.is_some())
    };

    if both_moved {
        process_round::<G>(room).await;
    }
}

async fn process_round<G: GameLogic>(room: &RoomState<G>) {
    // Get both player moves
    let (player1_move, player2_move) = {
        let players = room.players.read().await;
        let mut iter = players.values();
        let p1 = iter.next().unwrap();
        let p2 = iter.next().unwrap();
        (
            p1.current_move.clone().unwrap(),
            p2.current_move.clone().unwrap(),
        )
    };

    // Apply moves to game state
    {
        let mut game_state = room.game_state.write().await;

        if let Err(e) = G::make_move(&mut game_state, 0, &player1_move) {
            let _ = room.tx.send(GameMessage::Error {
                message: format!("Player 1 invalid move: {}", e),
            });
            return;
        }

        if let Err(e) = G::make_move(&mut game_state, 1, &player2_move) {
            let _ = room.tx.send(GameMessage::Error {
                message: format!("Player 2 invalid move: {}", e),
            });
            return;
        }

        // Send round result
        if let Some(result_msg) = G::get_round_result_message(&game_state, &player1_move, &player2_move) {
            let _ = room.tx.send(GameMessage::RoundResult { result: result_msg });
        }

        // Send updated game state
        let state_msg = G::get_state_message(&game_state);
        let _ = room.tx.send(GameMessage::GameState { state: state_msg });

        // Check if game is over
        if G::is_game_over(&game_state) {
            let game_over_msg = G::get_game_over_message(&game_state);
            let _ = room.tx.send(GameMessage::GameOver {
                result: game_over_msg,
            });
        }
    }

    // Clear moves for next round
    {
        let mut players = room.players.write().await;
        for player in players.values_mut() {
            player.current_move = None;
        }
    }

    // Increment round counter
    {
        let mut round = room.current_round.write().await;
        *round += 1;

        // Check max rounds
        if *round >= room.max_rounds {
            let game_state = room.game_state.read().await;
            let game_over_msg = G::get_game_over_message(&game_state);
            let _ = room.tx.send(GameMessage::GameOver {
                result: game_over_msg,
            });
        }
    }
}

#[derive(Deserialize)]
struct MoveRequest {
    #[serde(rename = "move")]
    player_move: String,
}
