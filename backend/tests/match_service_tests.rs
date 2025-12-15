use axel_tournament::{
    config::DatabaseConfig,
    db,
    models::{
        ProgrammingLanguage,
        matches::{MatchParticipantResult, MatchStatus},
    },
    services::{auth::AuthService, game, matches, submission, tournament},
};

async fn setup_test_db() -> axel_tournament::db::Database {
    let config = DatabaseConfig {
        url: "ws://127.0.0.1:8001".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: "test_match".to_string(),
        database: "test_match".to_string(),
    };
    db::connect(&config)
        .await
        .expect("Failed to connect to test database")
}

fn unique_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", prefix, timestamp)
}

#[tokio::test]
async fn test_match_create() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    // Create game
    let game = game::create_game(
        &db,
        unique_name("Game "),
        "Test game".to_string(),
        serde_json::json!({}),
        vec![ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();
    // Create 2 users and submissions
    let mut submission_ids = Vec::new();

    for i in 0..2 {
        let user_email = unique_name(&format!("user{}", i)) + "@test.com";
        let password_hash = auth_service.hash_password("password123").unwrap();
        let user = axel_tournament::services::user::create_user(
            &db,
            user_email,
            unique_name("user"),
            Some(password_hash),
            "US".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        let user_id = user.id.unwrap().id.to_string();
        let tournament_data = tournament::create_tournament(
            &db,
            game_id.clone(),
            unique_name("Tournament "),
            "Test tournament".to_string(),
            2,
            100,
            None,
            None,
        )
        .await
        .unwrap();
        let tournament_id = tournament_data.id.unwrap().id.to_string();
        let sub = submission::create_submission(
            &db,
            &user_id,
            &tournament_id,
            &game_id,
            ProgrammingLanguage::Rust,
            "fn main() {}".to_string(),
        )
        .await
        .unwrap();
        submission_ids.push(sub.id.unwrap().id.to_string());
    }
    // Create Match
    let match_data = matches::create_match(
        &db,
        None, // No tournament for this test, just friendly match context
        game_id.clone(),
        submission_ids.clone(),
    )
    .await;
    assert!(match_data.is_ok());
    let created_match = match_data.unwrap();

    assert_eq!(created_match.participants.len(), 2);
    assert_eq!(created_match.status, MatchStatus::Pending);
}

#[tokio::test]
async fn test_match_update_result() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    // Create game
    let game = game::create_game(
        &db,
        unique_name("Game "),
        "Test game".to_string(),
        serde_json::json!({}),
        vec![ProgrammingLanguage::Rust],
    )
    .await
    .unwrap();
    let game_id = game.id.unwrap().id.to_string();
    // Create user and submission
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let user = axel_tournament::services::user::create_user(
        &db,
        user_email,
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = user.id.unwrap().id.to_string();

    let tournament_data = tournament::create_tournament(
        &db,
        game_id.clone(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament_data.id.unwrap().id.to_string();
    let sub = submission::create_submission(
        &db,
        &user_id,
        &tournament_id,
        &game_id,
        ProgrammingLanguage::Rust,
        "fn main() {}".to_string(),
    )
    .await
    .unwrap();
    let sub_id = sub.id.unwrap().id.to_string();
    let match_data = matches::create_match(&db, None, game_id.clone(), vec![sub_id.clone()])
        .await
        .unwrap();
    let match_id = match_data.id.unwrap().id.to_string();
    // Update result
    let result = MatchParticipantResult {
        submission_id: sub_id.clone(),
        score: 100.0,
        rank: Some(1),
        is_winner: true,
        metadata: None,
    };
    let updated = matches::update_match_result(
        &db,
        &match_id,
        MatchStatus::Completed,
        vec![result],
        None,
        None,
        None,
    )
    .await;
    assert!(updated.is_ok());
    let u_match = updated.unwrap();
    assert_eq!(u_match.status, MatchStatus::Completed);
    assert_eq!(u_match.participants[0].score, Some(100.0));
    assert_eq!(u_match.participants[0].is_winner, true);
}
