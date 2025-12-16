mod common;

use axel_tournament::{
    config::DatabaseConfig,
    db,
    models::{CreateTournamentRequest, TournamentStatus, MatchGenerationType, MatchStatus},
    services::{auth::AuthService, game, tournament, user, submission, matches},
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
        match_generation_type: None,
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
        match_generation_type: None,
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
        match_generation_type: None,
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

#[tokio::test]
async fn test_start_tournament_all_vs_all() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);

    // Create game
    let game = game::create_game(
        &db,
        common::unique_name("MatchGen Game "),
        "Match generation test".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();

    // Create tournament with AllVsAll match generation (default)
    let tournament = tournament::create_tournament(
        &db,
        game_id.clone(),
        common::unique_name("AllVsAll Tournament "),
        "Test tournament".to_string(),
        2,
        10,
        None,
        None,
        Some(MatchGenerationType::AllVsAll),
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap().id.to_string();

    // Create 3 users
    let mut user_ids = Vec::new();
    for i in 0..3 {
        let password_hash = auth_service.hash_password("password123").unwrap();
        let user = user::create_user(
            &db,
            format!("{}@test.com", common::unique_name(&format!("player{}_", i))),
            common::unique_name(&format!("player{}_", i)),
            Some(password_hash),
            "US".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        user_ids.push(user.id.unwrap().id.to_string());
    }

    // Join tournament and create submissions
    for user_id in &user_ids {
        tournament::join_tournament(&db, &tournament_id, user_id)
            .await
            .unwrap();

        submission::create_submission(
            &db,
            user_id,
            &tournament_id,
            &game_id,
            axel_tournament::models::ProgrammingLanguage::Rust,
            "fn main() {}".to_string(),
        )
        .await
        .unwrap();
    }

    // Start tournament and generate matches
    let started_tournament = tournament::start_tournament(&db, &tournament_id)
        .await
        .unwrap();

    // Verify tournament status changed
    assert_eq!(started_tournament.status, TournamentStatus::Running);
    assert!(started_tournament.matches_generated);

    // Verify matches were created (3 players vs 3 players = 9 matches)
    let created_matches = matches::list_matches(&db, Some(tournament_id.clone()), None, None)
        .await
        .unwrap();
    assert_eq!(created_matches.len(), 9); // 3x3 = 9 matches

    // Verify all matches are in pending state
    for m in &created_matches {
        assert_eq!(m.status, MatchStatus::Pending);
        assert_eq!(m.participants.len(), 2);
    }
}

#[tokio::test]
async fn test_start_tournament_round_robin() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);

    // Create game
    let game = game::create_game(
        &db,
        common::unique_name("RoundRobin Game "),
        "Round robin test".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();

    // Create tournament with RoundRobin match generation
    let tournament = tournament::create_tournament(
        &db,
        game_id.clone(),
        common::unique_name("RoundRobin Tournament "),
        "Test tournament".to_string(),
        2,
        10,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap().id.to_string();

    // Create 4 users
    let mut user_ids = Vec::new();
    for i in 0..4 {
        let password_hash = auth_service.hash_password("password123").unwrap();
        let user = user::create_user(
            &db,
            format!("{}@test.com", common::unique_name(&format!("rrplayer{}_", i))),
            common::unique_name(&format!("rrplayer{}_", i)),
            Some(password_hash),
            "US".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        user_ids.push(user.id.unwrap().id.to_string());
    }

    // Join tournament and create submissions
    for user_id in &user_ids {
        tournament::join_tournament(&db, &tournament_id, user_id)
            .await
            .unwrap();

        submission::create_submission(
            &db,
            user_id,
            &tournament_id,
            &game_id,
            axel_tournament::models::ProgrammingLanguage::Rust,
            "fn main() {}".to_string(),
        )
        .await
        .unwrap();
    }

    // Start tournament and generate matches
    let started_tournament = tournament::start_tournament(&db, &tournament_id)
        .await
        .unwrap();

    // Verify tournament status changed
    assert_eq!(started_tournament.status, TournamentStatus::Running);
    assert!(started_tournament.matches_generated);

    // Verify matches were created (4 players, unique pairings = 6 matches)
    let created_matches = matches::list_matches(&db, Some(tournament_id.clone()), None, None)
        .await
        .unwrap();
    assert_eq!(created_matches.len(), 6); // 4 * (4-1) / 2 = 6 matches (no duplicates)
}

#[tokio::test]
async fn test_start_tournament_without_submissions_fails() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);

    // Create game
    let game = game::create_game(
        &db,
        common::unique_name("NoSub Game "),
        "No submissions test".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();

    // Create tournament
    let tournament = tournament::create_tournament(
        &db,
        game_id.clone(),
        common::unique_name("NoSub Tournament "),
        "Test tournament".to_string(),
        2,
        10,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap().id.to_string();

    // Create and join users but don't submit code
    for i in 0..2 {
        let password_hash = auth_service.hash_password("password123").unwrap();
        let user = user::create_user(
            &db,
            format!("{}@test.com", common::unique_name(&format!("nosub{}_", i))),
            common::unique_name(&format!("nosub{}_", i)),
            Some(password_hash),
            "US".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        let user_id = user.id.unwrap().id.to_string();

        tournament::join_tournament(&db, &tournament_id, &user_id)
            .await
            .unwrap();
    }

    // Try to start tournament - should fail because no submissions
    let result = tournament::start_tournament(&db, &tournament_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_start_tournament_not_enough_players_fails() {
    let db = setup_test_db().await;

    // Create game
    let game = game::create_game(
        &db,
        common::unique_name("MinPlayers Game "),
        "Min players test".to_string(),
        serde_json::json!({}),
        vec![axel_tournament::models::ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();

    // Create tournament requiring 5 minimum players
    let tournament = tournament::create_tournament(
        &db,
        game_id.clone(),
        common::unique_name("MinPlayers Tournament "),
        "Test tournament".to_string(),
        5,
        10,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament.id.clone().unwrap().id.to_string();

    // Try to start with 0 players - should fail
    let result = tournament::start_tournament(&db, &tournament_id).await;
    assert!(result.is_err());
}
