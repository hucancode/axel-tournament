use std::sync::Arc;
use judge::room::RoomManager;
use judge::games::TicTacToe;
use judge::games::Game;

#[tokio::test]
async fn test_room_creation_and_joining() {
    let room_manager = Arc::new(RoomManager::new());

    // Create a room
    let room = room_manager.create_room(
        "Test Room".to_string(),
        "tic-tac-toe".to_string(),
        "user:alice".to_string(),
        "Alice".to_string(),
        2,
        Some(30000),
    ).await;
    let room_id = room.id;

    // Verify room was created correctly
    let room_response = room_manager.get_room(&room_id).await.unwrap();
    assert_eq!(room_response.name, "Test Room");
    assert_eq!(room_response.game_id, "tic-tac-toe");
    assert_eq!(room_response.host_id, "user:alice");
    assert_eq!(room_response.host_username, "Alice");
    assert_eq!(room_response.players.len(), 1); // Host is automatically added

    let (room_response, is_reconnecting) = room_manager.join_room(&room_id, "user:bob".to_string(), "Bob".to_string()).await.unwrap();
    assert!(!is_reconnecting, "First join should not be reconnecting");
    assert_eq!(room_response.players.len(), 2); // Now has both players

    // Verify both players are in the room
    let alice_player = room_response.players.iter().find(|p| p.id == "user:alice").unwrap();
    let bob_player = room_response.players.iter().find(|p| p.id == "user:bob").unwrap();
    assert_eq!(alice_player.username, "Alice");
    assert_eq!(bob_player.username, "Bob");
}

#[tokio::test]
async fn test_game_reconnection_state_basic() {
    let game = TicTacToe::new();

    // Test reconnection state for a new game (not started yet)
    let alice_state = game.get_reconnect_state("user:alice");
    let bob_state = game.get_reconnect_state("user:bob");

    // For unstarted games, should return empty state
    assert!(alice_state.is_empty(), "Alice should get empty state for unstarted game");
    assert!(bob_state.is_empty(), "Bob should get empty state for unstarted game");
}

#[tokio::test]
async fn test_jwt_validation() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use judge::auth::{Claims, validate_jwt};

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
    assert_eq!(result.unwrap(), user_id);

    // Test invalid token
    let invalid_result = validate_jwt("invalid.token.here", secret);
    assert!(invalid_result.is_err());

    // Test wrong secret
    let wrong_secret_result = validate_jwt(&token, "wrong_secret");
    assert!(wrong_secret_result.is_err());
}

#[tokio::test]
async fn test_room_leave_functionality() {
    let room_manager = Arc::new(RoomManager::new());

    // Create room and join players
    let room = room_manager.create_room(
        "Test Room".to_string(),
        "tic-tac-toe".to_string(),
        "user:alice".to_string(),
        "Alice".to_string(),
        3,
        Some(30000),
    ).await;
    let room_id = room.id;

    room_manager.join_room(&room_id, "user:bob".to_string(), "Bob".to_string()).await.unwrap();
    room_manager.join_room(&room_id, "user:charlie".to_string(), "Charlie".to_string()).await.unwrap();

    // Verify all players are in room
    let room_response = room_manager.get_room(&room_id).await.unwrap();
    assert_eq!(room_response.players.len(), 3);

    // Bob leaves explicitly
    let leave_result = room_manager.leave_room(&room_id, "user:bob").await;
    assert!(matches!(leave_result, judge::room::LeaveResult::Left));

    // Verify Bob is removed
    let room_response = room_manager.get_room(&room_id).await.unwrap();
    assert_eq!(room_response.players.len(), 2);
    assert!(!room_response.players.iter().any(|p| p.id == "user:bob"));

    // Verify Alice and Charlie are still there
    assert!(room_response.players.iter().any(|p| p.id == "user:alice"));
    assert!(room_response.players.iter().any(|p| p.id == "user:charlie"));
}
