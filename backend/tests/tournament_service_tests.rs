mod common;

use axel_tournament::{
    config::DatabaseConfig,
    db,
    models::{CreateTournamentRequest, TournamentStatus},
    services::{auth::AuthService, game, tournament, user},
};
use validator::Validate;

async fn setup_test_db() -> axel_tournament::db::Database {
    let namespace = common::unique_name("test_tournament_");
    let config = DatabaseConfig {
        url: "ws://127.0.0.1:8001".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: namespace.clone(),
        database: namespace,
    };
    db::connect(&config)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_create_and_get_tournament() {
    let db = setup_test_db().await;
    let game = game::create_game(
        &db,
        common::unique_name("Tournament Game "),
        "Desc".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();
    let tournament = tournament::create_tournament(
        &db,
        game_id.clone(),
        common::unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        16,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap().id.to_string();
    let fetched = tournament::get_tournament(&db, &tournament_id)
        .await
        .unwrap();
    assert_eq!(fetched.name, tournament.name);
    assert_eq!(fetched.status, TournamentStatus::Registration);
}

#[tokio::test]
async fn test_update_tournament_status() {
    let db = setup_test_db().await;
    let game = game::create_game(
        &db,
        common::unique_name("Status Game "),
        "Desc".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();
    let tournament = tournament::create_tournament(
        &db,
        game_id,
        common::unique_name("Status Tournament "),
        "Test tournament".to_string(),
        2,
        8,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap().id.to_string();
    let updated = tournament::update_tournament(
        &db,
        &tournament_id,
        Some("Updated".to_string()),
        None,
        Some(TournamentStatus::Running),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.status, TournamentStatus::Running);
}

#[tokio::test]
async fn test_join_and_leave_tournament() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let game = game::create_game(
        &db,
        common::unique_name("Join Game "),
        "Desc".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();
    let tournament = tournament::create_tournament(
        &db,
        game_id,
        common::unique_name("Join Tournament "),
        "Test tournament".to_string(),
        2,
        8,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.unwrap().id.to_string();
    let password_hash = auth_service.hash_password("password123").unwrap();
    let user = user::create_user(
        &db,
        format!("{}@test.com", common::unique_name("player")),
        common::unique_name("player"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = user.id.unwrap().id.to_string();
    let participant = tournament::join_tournament(&db, &tournament_id, &user_id)
        .await
        .unwrap();
    assert_eq!(participant.user_id.id.to_string(), user_id);
    let participants = tournament::get_tournament_participants(&db, &tournament_id)
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    tournament::leave_tournament(&db, &tournament_id, &user_id)
        .await
        .unwrap();
    let after_leave = tournament::get_tournament_participants(&db, &tournament_id)
        .await
        .unwrap();
    assert!(after_leave.is_empty());
}

#[test]
fn test_create_tournament_request_validation() {
    let valid_request = CreateTournamentRequest {
        game_id: "game123".to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 2,
        max_players: 100,
        start_time: None,
        end_time: None,
    };
    assert!(valid_request.validate().is_ok());
    let low_min = CreateTournamentRequest {
        game_id: "game123".to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 1,
        max_players: 100,
        start_time: None,
        end_time: None,
    };
    assert!(low_min.validate().is_err());
    let high_max = CreateTournamentRequest {
        game_id: "game123".to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 2,
        max_players: 1000,
        start_time: None,
        end_time: None,
    };
    assert!(high_max.validate().is_err());
}

#[tokio::test]
async fn test_tournament_status_serialization() {
    let statuses = vec![
        TournamentStatus::Scheduled,
        TournamentStatus::Registration,
        TournamentStatus::Running,
        TournamentStatus::Completed,
        TournamentStatus::Cancelled,
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: TournamentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
