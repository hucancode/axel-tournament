mod db;

use std::sync::Arc;
use judge::services::room::RoomManager;
use judge::games::TicTacToe;
use judge::games::Game;

#[tokio::test]
async fn test_room_creation_and_joining() {
    let db = db::setup_test_db().await;
    let room_manager = Arc::new(RoomManager::new(db));

    // Create a room
    let room = room_manager.create_room(
        "Test Room".to_string(),
        "tic-tac-toe".to_string(),
        "user:alice".to_string(),
        2,
        Some(30000),
    ).await.unwrap();
    let room_id = room.id;

    // Verify room was created correctly
    let room_response = room_manager.get_room(&room_id).await.unwrap();
    assert_eq!(room_response.name, "Test Room");
    assert_eq!(room_response.game_id, "tic-tac-toe");
    assert_eq!(room_response.host_id, "user:alice");
    // Note: Room response only shows connected players (with active WebSockets)
    // Since we haven't connected via WebSocket yet, players list will be empty
    assert_eq!(room_response.players.len(), 0);

    let (room_response, is_reconnecting) = room_manager.join_room(&room_id, "user:bob".to_string()).await.unwrap();
    assert!(!is_reconnecting, "First join should not be reconnecting");
    // Still no WebSocket connections, so players list remains empty
    assert_eq!(room_response.players.len(), 0);
}

#[tokio::test]
async fn test_game_reconnection_state_basic() {
    let game = TicTacToe::new();

    // Test reconnection state for a new game (not started yet)
    let alice_state = game.get_event_source("user:alice");
    let bob_state = game.get_event_source("user:bob");

    // For unstarted games, should return empty state
    assert!(alice_state.is_empty(), "Alice should get empty state for unstarted game");
    assert!(bob_state.is_empty(), "Bob should get empty state for unstarted game");
}

#[tokio::test]
async fn test_jwt_validation() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use judge::middleware::auth::{Claims, validate_jwt};

    let secret = "test_secret_key";
    let user_id = "user:test123";

    // Create a valid JWT token
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ).unwrap();

    // Test valid token
    let result = validate_jwt(&token, secret);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().sub, user_id);

    // Test invalid token
    let invalid_result = validate_jwt("invalid.token.here", secret);
    assert!(invalid_result.is_err());

    // Test wrong secret
    let wrong_secret_result = validate_jwt(&token, "wrong_secret");
    assert!(wrong_secret_result.is_err());
}

#[tokio::test]
async fn test_room_leave_functionality() {
    let db = db::setup_test_db().await;
    let room_manager = Arc::new(RoomManager::new(db));

    // Create room and join players
    let room = room_manager.create_room(
        "Test Room".to_string(),
        "tic-tac-toe".to_string(),
        "user:alice".to_string(),
        3,
        Some(30000),
    ).await.unwrap();
    let room_id = room.id;

    room_manager.join_room(&room_id, "user:bob".to_string()).await.unwrap();
    room_manager.join_room(&room_id, "user:charlie".to_string()).await.unwrap();

    // Bob leaves explicitly
    let leave_result = room_manager.leave_room(&room_id, "user:bob").await;
    assert!(matches!(leave_result, judge::services::room::LeaveResult::Left));
}
