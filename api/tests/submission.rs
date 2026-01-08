// Unit tests for submission service logic
use api::{
    config::Config,
    db,
    models::{CreateSubmissionRequest, ProgrammingLanguage},
    services::{auth::AuthService, submission, tournament},
};
use validator::Validate;

async fn setup_test_db() -> api::db::Database {
    let config = Config::from_env();
    db::connect(&config.database)
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

// Use hardcoded game IDs (games are now maintained by developers)
const TEST_GAME_ID: &str = "rock-paper-scissors";

#[tokio::test]
async fn test_submission_create() {
    let db = setup_test_db().await;
    // Create a user first
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = api::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = created_user.id.unwrap();
    // Use hardcoded game (games are now maintained by developers)
    // Create a tournament
    let tournament_data = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament_data.id.unwrap();
    // Join tournament before submitting
    tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();
    // Create a submission
    let code = "fn main() { println!(\"Hello\"); }";
    let submission = submission::create_submission(
        &db,
        user_id.clone(),
        tournament_id.clone(),
        TEST_GAME_ID.to_string(),
        ProgrammingLanguage::Rust,
        code.to_string(),
    )
    .await;
    assert!(submission.is_ok());
    let created_submission = submission.unwrap();
    assert_eq!(created_submission.language, ProgrammingLanguage::Rust);
}

#[tokio::test]
async fn test_submission_get() {
    let db = setup_test_db().await;
    // Create user and tournament
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = api::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = created_user.id.unwrap();
    // Use hardcoded game (games are now maintained by developers)
    let tournament_data = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament_data.id.unwrap();
    // Join tournament before submitting
    tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();
    // Create a submission
    let code = "fn main() {}";
    let created_submission = submission::create_submission(
        &db,
        user_id.clone(),
        tournament_id.clone(),
        TEST_GAME_ID.to_string(),
        ProgrammingLanguage::Rust,
        code.to_string(),
    )
    .await
    .unwrap();
    let submission_id = created_submission.id.unwrap();
    // Get the submission
    let fetched_submission = submission::get_submission(&db, submission_id).await;
    assert!(fetched_submission.is_ok());
    let fetched = fetched_submission.unwrap();
    assert_eq!(fetched.language, ProgrammingLanguage::Rust);
}

#[tokio::test]
async fn test_submission_list_by_user() {
    let db = setup_test_db().await;
    // Create user and tournament
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user_email = unique_name("user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = api::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = created_user.id.unwrap();
    // Use hardcoded game (games are now maintained by developers)
    let tournament_data = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament_data.id.unwrap();
    // Join tournament before submitting
    tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();
    // Create multiple submissions
    submission::create_submission(
        &db,
        user_id.clone(),
        tournament_id.clone(),
        TEST_GAME_ID.to_string(),
        ProgrammingLanguage::Rust,
        "fn main() { println!(\"1\"); }".to_string(),
    )
    .await
    .unwrap();
    submission::create_submission(
        &db,
        user_id.clone(),
        tournament_id.clone(),
        TEST_GAME_ID.to_string(),
        ProgrammingLanguage::Rust,
        "fn main() { println!(\"2\"); }".to_string(),
    )
    .await
    .unwrap();
    // List submissions
    let submissions = submission::list_user_submissions(&db, user_id, None).await;
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

#[tokio::test]
async fn test_submission_workflow() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);

    // Create user
    let user_email = unique_name("workflow_user") + "@test.com";
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = api::services::user::create_user(
        &db,
        user_email.clone(),
        unique_name("workflow_user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = created_user.id.unwrap();

    // Create tournament
    let tournament_data = tournament::create_tournament(
        &db,
        TEST_GAME_ID.to_string(),
        unique_name("Workflow Tournament "),
        "Test tournament".to_string(),
        2,
        100,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let tournament_id = tournament_data.id.unwrap();

    // Join tournament
    tournament::join_tournament(&db, tournament_id.clone(), user_id.clone())
        .await
        .unwrap();

    // Create submission
    let code = "fn main() { println!(\"hello\"); }";
    let created_submission = submission::create_submission(
        &db,
        user_id.clone(),
        tournament_id.clone(),
        TEST_GAME_ID.to_string(),
        ProgrammingLanguage::Rust,
        code.to_string(),
    )
    .await
    .unwrap();

    let submission_id = created_submission.id.unwrap();

    // Get single submission
    let fetched_submission = submission::get_submission(&db, submission_id.clone())
        .await
        .unwrap();
    assert_eq!(fetched_submission.code, code);
    assert_eq!(fetched_submission.language, ProgrammingLanguage::Rust);

    // List user submissions
    let user_submissions = submission::list_user_submissions(&db, user_id, None)
        .await
        .unwrap();
    assert!(!user_submissions.is_empty());
    assert!(
        user_submissions
            .iter()
            .any(|s| s.id == Some(submission_id.clone()))
    );
}
