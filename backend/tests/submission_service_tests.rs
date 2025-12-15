// Unit tests for submission service logic
use axel_tournament::{
    config::DatabaseConfig,
    db,
    models::{ProgrammingLanguage, CreateSubmissionRequest},
    services::{submission, game, tournament, auth::AuthService},
};
use validator::Validate;

async fn setup_test_db() -> axel_tournament::db::Database {
    let config = DatabaseConfig {
        url: "ws://127.0.0.1:8001".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: "test_submission".to_string(),
        database: "test_submission".to_string(),
    };

    db::connect(&config).await.expect("Failed to connect to test database")
}

fn unique_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", prefix, timestamp)
}

#[tokio::test]
async fn test_submission_create() {
    let db = setup_test_db().await;

    // Create a user first
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = axel_tournament::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    ).await.unwrap();
    let user_id = created_user.id.unwrap().id.to_string();

    // Create a game
    let game = game::create_game(
        &db,
        unique_name("Game "),
        "Test game".to_string(),
        serde_json::json!({}),
        vec![ProgrammingLanguage::Rust],
    ).await.unwrap();
    let game_id = game.id.unwrap().id.to_string();

    // Create a tournament
    let tournament_data = tournament::create_tournament(
        &db,
        game_id.clone(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
    ).await.unwrap();
    let tournament_id = tournament_data.id.unwrap().id.to_string();

    // Create a submission
    let code = "fn main() { println!(\"Hello\"); }";
    let submission = submission::create_submission(
        &db,
        &user_id,
        &tournament_id,
        &game_id,
        ProgrammingLanguage::Rust,
        code.to_string(),
    ).await;

    assert!(submission.is_ok());
    let created_submission = submission.unwrap();

    assert_eq!(created_submission.language, ProgrammingLanguage::Rust);
    assert!(created_submission.file_path.contains(".rs"));
}

#[tokio::test]
async fn test_submission_get() {
    let db = setup_test_db().await;

    // Create user, game, tournament
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = axel_tournament::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    ).await.unwrap();
    let user_id = created_user.id.unwrap().id.to_string();

    let game = game::create_game(
        &db,
        unique_name("Game "),
        "Test game".to_string(),
        serde_json::json!({}),
        vec![ProgrammingLanguage::Rust],
    ).await.unwrap();
    let game_id = game.id.unwrap().id.to_string();

    let tournament_data = tournament::create_tournament(
        &db,
        game_id.clone(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
    ).await.unwrap();
    let tournament_id = tournament_data.id.unwrap().id.to_string();

    // Create a submission
    let code = "fn main() {}";
    let created_submission = submission::create_submission(
        &db,
        &user_id,
        &tournament_id,
        &game_id,
        ProgrammingLanguage::Rust,
        code.to_string(),
    ).await.unwrap();
    let submission_id = created_submission.id.unwrap().id.to_string();

    // Get the submission
    let fetched_submission = submission::get_submission(&db, &submission_id).await;

    assert!(fetched_submission.is_ok());
    let fetched = fetched_submission.unwrap();
    assert_eq!(fetched.language, ProgrammingLanguage::Rust);
}

#[tokio::test]
async fn test_submission_list_by_user() {
    let db = setup_test_db().await;

    // Create user, game, tournament
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = axel_tournament::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    ).await.unwrap();
    let user_id = created_user.id.unwrap().id.to_string();

    let game = game::create_game(
        &db,
        unique_name("Game "),
        "Test game".to_string(),
        serde_json::json!({}),
        vec![ProgrammingLanguage::Rust],
    ).await.unwrap();
    let game_id = game.id.unwrap().id.to_string();

    let tournament_data = tournament::create_tournament(
        &db,
        game_id.clone(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
    ).await.unwrap();
    let tournament_id = tournament_data.id.unwrap().id.to_string();

    // Create multiple submissions
    submission::create_submission(
        &db,
        &user_id,
        &tournament_id,
        &game_id,
        ProgrammingLanguage::Rust,
        "fn main() { println!(\"1\"); }".to_string(),
    ).await.unwrap();

    submission::create_submission(
        &db,
        &user_id,
        &tournament_id,
        &game_id,
        ProgrammingLanguage::Rust,
        "fn main() { println!(\"2\"); }".to_string(),
    ).await.unwrap();

    // List submissions
    let submissions = submission::list_user_submissions(&db, &user_id, None).await;

    assert!(submissions.is_ok());
    assert!(submissions.unwrap().len() >= 2);
}

#[tokio::test]
async fn test_submission_request_validation() {
    // Valid request
    let valid = CreateSubmissionRequest {
        tournament_id: "tournament123".to_string(),
        language: "rust".to_string(),
        code: "fn main() {}".to_string(),
    };
    assert!(valid.validate().is_ok());

    // Empty code - should fail
    let empty_code = CreateSubmissionRequest {
        tournament_id: "tournament123".to_string(),
        language: "rust".to_string(),
        code: "".to_string(),
    };
    assert!(empty_code.validate().is_err());

    // Code too long (> 1MB)
    let long_code = CreateSubmissionRequest {
        tournament_id: "tournament123".to_string(),
        language: "rust".to_string(),
        code: "x".repeat(1_100_000),
    };
    assert!(long_code.validate().is_err());
}
