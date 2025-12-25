// Unit tests for game service logic
use axel_tournament::{
    db,
    models::{CreateGameRequest, GameType, ProgrammingLanguage, UpdateGameRequest},
    services::game,
};
use surrealdb::sql::Thing;
use validator::Validate;

async fn setup_test_db() -> axel_tournament::db::Database {
    let config = axel_tournament::config::Config::from_env()
        .expect("Failed to load config from environment");
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

const DEFAULT_GAME_CODE: &str = "fn main() {}";
const DEFAULT_ROUNDS_PER_MATCH: u32 = 3;
const DEFAULT_REPETITIONS: u32 = 1;
const DEFAULT_TIMEOUT_MS: u32 = 2000;
const DEFAULT_CPU_LIMIT: f64 = 1.0;
const DEFAULT_TURN_TIMEOUT_MS: u64 = 200;
const DEFAULT_MEMORY_LIMIT_MB: u64 = 64;

fn default_owner_id() -> String {
    "user:owner".to_string()
}

#[tokio::test]
async fn test_game_create() {
    let db = setup_test_db().await;
    let name = unique_name("Battle Arena ");
    let game = game::create_game(
        &db,
        name.clone(),
        "A competitive battle arena game".to_string(),
        GameType::Automated,
        vec![
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await;
    assert!(game.is_ok());
    let created_game = game.unwrap();
    assert_eq!(created_game.name, name);
    assert_eq!(created_game.description, "A competitive battle arena game");
    assert_eq!(created_game.is_active, true);
    assert_eq!(created_game.supported_languages.len(), 3);
}

#[tokio::test]
async fn test_game_get() {
    let db = setup_test_db().await;
    // Create a game first
    let name = unique_name("Test Game ");
    let created_game = game::create_game(
        &db,
        name.clone(),
        "Test Description".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    let game_id = created_game.id.unwrap();
    // Get the game
    let fetched_game = game::get_game(&db, game_id.clone()).await;
    assert!(fetched_game.is_ok());
    let fetched = fetched_game.unwrap();
    assert_eq!(fetched.name, name);
}

#[tokio::test]
async fn test_game_get_nonexistent() {
    let db = setup_test_db().await;
    let result = game::get_game(&db, Thing::from(("game", "nonexistent_id"))).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_game_list_all() {
    let db = setup_test_db().await;
    // Create multiple games
    game::create_game(
        &db,
        unique_name("Game 1 "),
        "First game".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    game::create_game(
        &db,
        unique_name("Game 2 "),
        "Second game".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Go],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Go,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    let games = game::list_games(&db, false).await;
    assert!(games.is_ok());
    assert!(games.unwrap().len() >= 2);
}

#[tokio::test]
async fn test_game_list_active_only() {
    let db = setup_test_db().await;
    // Create an active game
    let active_name = unique_name("Active Game ");
    let _active_game = game::create_game(
        &db,
        active_name.clone(),
        "Active".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    // Create and deactivate a game
    let inactive_game = game::create_game(
        &db,
        unique_name("Inactive Game "),
        "Inactive".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::C],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::C,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    let inactive_id = inactive_game.id.unwrap();
    game::update_game(
        &db,
        inactive_id,
        None,
        None,
        Some(vec![]),
        Some(false),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
        .await
        .unwrap();
    // List only active games
    let active_games = game::list_games(&db, true).await.unwrap();
    // Should have at least the active game
    assert!(active_games.iter().any(|g| g.name == active_name));
}

#[tokio::test]
async fn test_game_update_name() {
    let db = setup_test_db().await;
    let created_game = game::create_game(
        &db,
        unique_name("Original Name "),
        "Description".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    let game_id = created_game.id.unwrap();
    // Update the name
    let new_name = unique_name("Updated Name ");
    let updated_game = game::update_game(
        &db,
        game_id.clone(),
        Some(new_name.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    assert!(updated_game.is_ok());
    let updated = updated_game.unwrap();
    assert_eq!(updated.name, new_name);
    assert_eq!(updated.description, "Description"); // Unchanged
}

#[tokio::test]
async fn test_game_update_description() {
    let db = setup_test_db().await;
    let created_game = game::create_game(
        &db,
        unique_name("Game Name "),
        "Original Description".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    let game_id = created_game.id.unwrap();
    // Update the description
    let updated_game = game::update_game(
        &db,
        game_id,
        None,
        Some("New Description".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(updated_game.description, "New Description");
}

#[tokio::test]
async fn test_game_update_deactivate() {
    let db = setup_test_db().await;
    let created_game = game::create_game(
        &db,
        unique_name("Game "),
        "Desc".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    assert_eq!(created_game.is_active, true);
    let game_id = created_game.id.unwrap();
    // Deactivate
    let updated_game = game::update_game(
        &db,
        game_id.clone(),
        None,
        None,
        None,
        Some(false),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
        .await
        .unwrap();
    assert_eq!(updated_game.is_active, false);
}

#[tokio::test]
async fn test_game_delete() {
    let db = setup_test_db().await;
    let created_game = game::create_game(
        &db,
        unique_name("To Delete "),
        "Will be deleted".to_string(),
        GameType::Automated,
        vec![ProgrammingLanguage::Rust],
        default_owner_id(),
        DEFAULT_GAME_CODE.to_string(),
        ProgrammingLanguage::Rust,
        None,
        DEFAULT_ROUNDS_PER_MATCH,
        DEFAULT_REPETITIONS,
        DEFAULT_TIMEOUT_MS,
        DEFAULT_CPU_LIMIT,
        DEFAULT_TURN_TIMEOUT_MS,
        DEFAULT_MEMORY_LIMIT_MB,
    )
    .await
    .unwrap();
    let game_id = created_game.id.unwrap();
    // Delete the game
    let result = game::delete_game(&db, game_id.clone()).await;
    assert!(result.is_ok());
    // Try to fetch it - should fail
    let fetch_result = game::get_game(&db, game_id).await;
    assert!(fetch_result.is_err());
}

#[tokio::test]
async fn test_game_request_validation() {
    // Valid request
    let valid = CreateGameRequest {
        name: "Valid Game".to_string(),
        description: "A valid game description".to_string(),
        game_type: GameType::Automated,
        supported_languages: vec![ProgrammingLanguage::Rust],
        game_code: DEFAULT_GAME_CODE.to_string(),
        game_language: ProgrammingLanguage::Rust,
        frontend_code: None,
        rounds_per_match: DEFAULT_ROUNDS_PER_MATCH,
        repetitions: DEFAULT_REPETITIONS,
        timeout_ms: DEFAULT_TIMEOUT_MS,
        cpu_limit: DEFAULT_CPU_LIMIT,
        turn_timeout_ms: DEFAULT_TURN_TIMEOUT_MS,
        memory_limit_mb: DEFAULT_MEMORY_LIMIT_MB,
    };
    assert!(valid.validate().is_ok());
    // Empty name - should fail
    let empty_name = CreateGameRequest {
        name: "".to_string(),
        description: "Description".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
        game_code: DEFAULT_GAME_CODE.to_string(),
        game_language: ProgrammingLanguage::Rust,
        game_type: GameType::Automated,
        frontend_code: None,
        rounds_per_match: DEFAULT_ROUNDS_PER_MATCH,
        repetitions: DEFAULT_REPETITIONS,
        timeout_ms: DEFAULT_TIMEOUT_MS,
        cpu_limit: DEFAULT_CPU_LIMIT,
        turn_timeout_ms: DEFAULT_TURN_TIMEOUT_MS,
        memory_limit_mb: DEFAULT_MEMORY_LIMIT_MB,
    };
    assert!(empty_name.validate().is_err());
    // Name too long
    let long_name = CreateGameRequest {
        name: "a".repeat(101),
        description: "Description".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
        game_code: DEFAULT_GAME_CODE.to_string(),
        game_language: ProgrammingLanguage::Rust,
        game_type: GameType::Automated,
        frontend_code: None,
        rounds_per_match: DEFAULT_ROUNDS_PER_MATCH,
        repetitions: DEFAULT_REPETITIONS,
        timeout_ms: DEFAULT_TIMEOUT_MS,
        cpu_limit: DEFAULT_CPU_LIMIT,
        turn_timeout_ms: DEFAULT_TURN_TIMEOUT_MS,
        memory_limit_mb: DEFAULT_MEMORY_LIMIT_MB,
    };
    assert!(long_name.validate().is_err());
    // Empty description
    let empty_desc = CreateGameRequest {
        name: "Name".to_string(),
        description: "".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
        game_code: DEFAULT_GAME_CODE.to_string(),
        game_language: ProgrammingLanguage::Rust,
        game_type: GameType::Automated,
        frontend_code: None,
        rounds_per_match: DEFAULT_ROUNDS_PER_MATCH,
        repetitions: DEFAULT_REPETITIONS,
        timeout_ms: DEFAULT_TIMEOUT_MS,
        cpu_limit: DEFAULT_CPU_LIMIT,
        turn_timeout_ms: DEFAULT_TURN_TIMEOUT_MS,
        memory_limit_mb: DEFAULT_MEMORY_LIMIT_MB,
    };
    assert!(empty_desc.validate().is_err());
}

#[tokio::test]
async fn test_game_update_request_validation() {
    // All fields optional
    let empty_update = UpdateGameRequest {
        name: None,
        description: None,
        supported_languages: None,
        is_active: None,
        game_code: None,
        game_language: None,
        game_type: None,
        frontend_code: None,
        rounds_per_match: None,
        repetitions: None,
        timeout_ms: None,
        cpu_limit: None,
        turn_timeout_ms: None,
        memory_limit_mb: None,
    };
    assert!(empty_update.validate().is_ok());
    // Valid partial update
    let partial = UpdateGameRequest {
        name: Some("New Name".to_string()),
        description: None,
        supported_languages: None,
        is_active: Some(false),
        game_code: None,
        game_language: None,
        game_type: None,
        frontend_code: None,
        rounds_per_match: None,
        repetitions: None,
        timeout_ms: None,
        cpu_limit: None,
        turn_timeout_ms: None,
        memory_limit_mb: None,
    };
    assert!(partial.validate().is_ok());
    // Invalid: name too long
    let invalid_name = UpdateGameRequest {
        name: Some("a".repeat(101)),
        description: None,
        supported_languages: None,
        is_active: None,
        game_code: None,
        game_language: None,
        game_type: None,
        frontend_code: None,
        rounds_per_match: None,
        repetitions: None,
        timeout_ms: None,
        cpu_limit: None,
        turn_timeout_ms: None,
        memory_limit_mb: None,
    };
    assert!(invalid_name.validate().is_err());
}
