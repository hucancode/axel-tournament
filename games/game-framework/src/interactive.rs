use serde::{Deserialize, Serialize};

/// WebSocket message types for interactive games
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameMessage {
    PlayerJoined {
        player_id: String,
        player_name: String,
    },
    PlayerLeft {
        player_id: String,
    },
    GameStarted {
        players: Vec<String>,
    },
    GameMove {
        player_id: String,
        #[serde(rename = "move")]
        move_data: serde_json::Value,
    },
    GameState {
        state: serde_json::Value,
    },
    GameOver {
        winner: Option<String>,
        final_state: serde_json::Value,
    },
    ChatMessage {
        player_id: String,
        message: String,
    },
    Error {
        message: String,
    },
}

/// Room status for interactive games
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RoomStatus {
    Waiting,
    Playing,
    Finished,
}

/// Trait for interactive game servers that handle real-time gameplay
pub trait InteractiveGameServer {
    /// Get the current state of a room
    fn get_room_state(&self, room_id: &str) -> Option<RoomStatus>;

    /// Handle a player connection to a room
    fn handle_player_join(&mut self, room_id: &str, player_id: &str) -> Result<(), String>;

    /// Handle a player disconnection from a room
    fn handle_player_leave(&mut self, room_id: &str, player_id: &str);

    /// Process a game move from a player
    fn process_move(
        &mut self,
        room_id: &str,
        player_id: &str,
        move_data: serde_json::Value,
    ) -> Result<GameMessage, String>;
}
