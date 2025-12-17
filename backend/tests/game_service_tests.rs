// Unit tests for game service logic
use axel_tournament::{
    config::DatabaseConfig,
    db,
    models::{CreateGameRequest, ProgrammingLanguage, UpdateGameRequest},
    services::game,
};
use surrealdb::sql::Thing;
use validator::Validate;

async fn setup_test_db() -> axel_tournament::db::Database {
    let config = DatabaseConfig {
        url: "ws://127.0.0.1:8001".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: "test_game".to_string(),
        database: "test_game".to_string(),
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
async fn test_game_create() {
    let db = setup_test_db().await;
    let name = unique_name("Battle Arena ");
    let game = game::create_game(
        &db,
        name.clone(),
        "A competitive battle arena game".to_string(),
        vec![
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::C,
        ],
        None,
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
        vec![ProgrammingLanguage::Rust],
        None,
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
        vec![ProgrammingLanguage::Rust],
        None,
    )
    .await
    .unwrap();
    game::create_game(
        &db,
        unique_name("Game 2 "),
        "Second game".to_string(),
        vec![ProgrammingLanguage::Go],
        None,
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
        vec![ProgrammingLanguage::Rust],
        None,
    )
    .await
    .unwrap();
    // Create and deactivate a game
    let inactive_game = game::create_game(
        &db,
        unique_name("Inactive Game "),
        "Inactive".to_string(),
        vec![ProgrammingLanguage::C],
        None,
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
        vec![ProgrammingLanguage::Rust],
        None,
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
        vec![ProgrammingLanguage::Rust],
        None,
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
        vec![ProgrammingLanguage::Rust],
        None,
    )
    .await
    .unwrap();
    assert_eq!(created_game.is_active, true);
    let game_id = created_game.id.unwrap();
    // Deactivate
    let updated_game = game::update_game(&db, game_id.clone(), None, None, None, Some(false))
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
        vec![ProgrammingLanguage::Rust],
        None,
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
        supported_languages: vec![ProgrammingLanguage::Rust],
    };
    assert!(valid.validate().is_ok());
    // Empty name - should fail
    let empty_name = CreateGameRequest {
        name: "".to_string(),
        description: "Description".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
    };
    assert!(empty_name.validate().is_err());
    // Name too long
    let long_name = CreateGameRequest {
        name: "a".repeat(101),
        description: "Description".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
    };
    assert!(long_name.validate().is_err());
    // Empty description
    let empty_desc = CreateGameRequest {
        name: "Name".to_string(),
        description: "".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
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
    };
    assert!(empty_update.validate().is_ok());
    // Valid partial update
    let partial = UpdateGameRequest {
        name: Some("New Name".to_string()),
        description: None,
        supported_languages: None,
        is_active: Some(false),
    };
    assert!(partial.validate().is_ok());
    // Invalid: name too long
    let invalid_name = UpdateGameRequest {
        name: Some("a".repeat(101)),
        description: None,
        supported_languages: None,
        is_active: None,
    };
    assert!(invalid_name.validate().is_err());
}
