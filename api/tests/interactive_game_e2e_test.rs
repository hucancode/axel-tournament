use axel_tournament::{
    db,
    services::{auth::AuthService, game, room, interactive_match},
    models::{GameType, ProgrammingLanguage, User, UserRole},
};
use std::sync::Arc;

mod common;

const INTERACTIVE_GAME_CODE: &str = include_str!("../../games/tic_tac_toe/server.rs");
const INTERACTIVE_FRONTEND_CODE: &str = include_str!("../../games/tic_tac_toe/client.html");

#[tokio::test]
async fn test_interactive_game_flow() {
    // Setup test database
    let config = axel_tournament::config::Config::from_env().unwrap();
    let db = db::connect(&config.database).await.unwrap();

    // Create test users with unique names
    let auth_service = Arc::new(AuthService::new("test_secret".to_string(), 3600));

    let user1_email = format!("{}@test.com", common::unique_name("player1"));
    let user1_username = common::unique_name("player1");
    let user2_email = format!("{}@test.com", common::unique_name("player2"));
    let user2_username = common::unique_name("player2");

    let user1 = User {
        id: None,
        email: user1_email,
        username: user1_username,
        password_hash: Some(auth_service.hash_password("password").unwrap()),
        role: UserRole::Player,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: surrealdb::sql::Datetime::default(),
        updated_at: surrealdb::sql::Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };

    let user2 = User {
        id: None,
        email: user2_email,
        username: user2_username,
        password_hash: Some(auth_service.hash_password("password").unwrap()),
        role: UserRole::Player,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: surrealdb::sql::Datetime::default(),
        updated_at: surrealdb::sql::Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };

    let created_user1: Option<User> = db.create("user").content(user1).await.unwrap();
    let created_user2: Option<User> = db.create("user").content(user2).await.unwrap();

    let user1_id = created_user1.unwrap().id.unwrap().to_string();
    let user2_id = created_user2.unwrap().id.unwrap().to_string();

    // Create interactive game
    let game_name = common::unique_name("Test Interactive Game");
    let game = game::create_game(
        &db,
        game_name,
        "Test interactive tic-tac-toe game".to_string(),
        GameType::Interactive,
        vec![ProgrammingLanguage::Rust],
        user1_id.clone(),
        INTERACTIVE_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        Some(INTERACTIVE_FRONTEND_CODE.to_string()),
        1,
        1,
        5000,
        0.5,
        2000,
        64,
    ).await.unwrap();

    // Create room
    let room_name = common::unique_name("Test Room");
    let room = room::create_room(
        &db,
        game.id.unwrap().to_string(),
        user1_id.clone(),
        room_name,
        2,
    ).await.unwrap();

    let room_id = room.id.unwrap();

    // Player 2 joins room
    let room = room::join_room(&db, room_id.clone(), user2_id.clone()).await.unwrap();
    assert_eq!(room.players.len(), 2);

    // Start game
    let room = room::start_game(&db, room_id.clone(), user1_id.clone()).await.unwrap();
    assert_eq!(room.status, axel_tournament::models::RoomStatus::Playing);

    // Verify match was created
    let matches: Vec<axel_tournament::models::Match> = db
        .query("SELECT * FROM match WHERE room_id = $room_id")
        .bind(("room_id", room_id.clone()))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert_eq!(matches.len(), 1);
    let match_data = &matches[0];
    assert_eq!(match_data.status, axel_tournament::models::MatchStatus::Running);
    assert_eq!(match_data.participants.len(), 2);

    // Complete match
    let completed_match = interactive_match::complete_interactive_match(
        &db,
        match_data.id.as_ref().unwrap().to_string(),
        vec![1.0, 0.0], // Player 1 wins
        Some(serde_json::json!({"winner": "player1"})),
    ).await.unwrap();

    assert_eq!(completed_match.status, axel_tournament::models::MatchStatus::Completed);
    assert_eq!(completed_match.participants[0].score, Some(1.0));
    assert_eq!(completed_match.participants[1].score, Some(0.0));
}
