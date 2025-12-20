mod common;

use axum::http::{self, StatusCode};
use serde_json::json;

#[tokio::test]
async fn game_setter_can_upload_game_code() {
    let app = common::setup_app(&common::unique_name("game_code_")).await;
    let game_setter_token = common::game_setter_token(&app).await;

    // Create a game as game setter
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust", "go"]
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Upload game code in Rust
    let game_code = r#"
        fn main() {
            println!("[100, 90, 80]");
        }
    "#;

    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "rust",
            "code_content": game_code
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(upload_status, StatusCode::OK);
    assert!(
        upload_body["message"]
            .as_str()
            .unwrap()
            .contains("successfully")
    );

    // Verify game was updated
    let (get_status, get_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/games/{}", game_id),
        None,
        None,
    )
    .await;

    assert_eq!(get_status, StatusCode::OK);
    assert!(get_body["game_code"].as_str().is_some());
    assert_eq!(get_body["game_language"], "rust");
}

#[tokio::test]
async fn game_code_upload_validates_language() {
    let app = common::setup_app(&common::unique_name("game_code_")).await;
    let game_setter_token = common::game_setter_token(&app).await;

    // Create a game that only supports Rust
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]  // Only Rust
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Try to upload Go code (not supported)
    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "go",
            "code_content": "package main\nfunc main() {}"
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(upload_status, StatusCode::BAD_REQUEST);
    assert!(
        upload_body["error"]
            .as_str()
            .unwrap()
            .contains("not supported")
    );
}

#[tokio::test]
async fn game_code_upload_rejects_invalid_language() {
    let app = common::setup_app(&common::unique_name("game_code_")).await;
    let game_setter_token = common::game_setter_token(&app).await;

    // Create a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Try to upload with invalid language
    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "javascript",  // Not a valid language
            "code_content": "console.log('test')"
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(upload_status, StatusCode::BAD_REQUEST);
    assert!(
        upload_body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid language")
    );
}

#[tokio::test]
async fn game_code_upload_checks_ownership() {
    let app = common::setup_app(&common::unique_name("game_code_")).await;
    let game_setter1_token = common::game_setter_token(&app).await;
    let game_setter2_token = common::game_setter_token(&app).await;

    // Game setter 1 creates a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter1_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Game setter 2 tries to upload game code (should fail)
    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "rust",
            "code_content": "fn main() {}"
        })),
        Some(&game_setter2_token),
    )
    .await;

    assert_eq!(upload_status, StatusCode::FORBIDDEN);
    assert!(
        upload_body["error"]
            .as_str()
            .unwrap()
            .contains("permission")
    );
}

#[tokio::test]
async fn admin_can_upload_game_code_to_any_game() {
    let app = common::setup_app(&common::unique_name("game_code_")).await;
    let game_setter_token = common::game_setter_token(&app).await;
    let admin_token = common::admin_token(&app).await;

    // Game setter creates a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Admin uploads game code (should succeed)
    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "rust",
            "code_content": "fn main() { println!(\"[100, 100]\"); }"
        })),
        Some(&admin_token),
    )
    .await;

    assert_eq!(upload_status, StatusCode::OK);
    assert!(
        upload_body["message"]
            .as_str()
            .unwrap()
            .contains("successfully")
    );
}

#[tokio::test]
async fn game_setter_can_list_only_their_games() {
    let app = common::setup_app(&common::unique_name("game_list_")).await;
    let game_setter1_token = common::game_setter_token(&app).await;
    let game_setter2_token = common::game_setter_token(&app).await;

    // Game setter 1 creates 2 games
    for i in 1..=2 {
        let (status, _) = common::json_request(
            &app,
            http::Method::POST,
            "/api/game-setter/games",
            Some(json!({
                "name": format!("GS1 Game {} {}", i, common::unique_name("")),
                "description": "Test",
                "supported_languages": ["rust"]
            })),
            Some(&game_setter1_token),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    // Game setter 2 creates 1 game
    let (status, _) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("GS2 Game {}", common::unique_name("")),
            "description": "Test",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter2_token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Game setter 1 lists their games (should see only 2)
    let (list_status, list_body) = common::json_request(
        &app,
        http::Method::GET,
        "/api/game-setter/games",
        None,
        Some(&game_setter1_token),
    )
    .await;

    assert_eq!(list_status, StatusCode::OK);
    assert!(list_body.is_array());
    let games = list_body.as_array().unwrap();
    assert_eq!(games.len(), 2);

    // Verify all games belong to game setter 1
    for game in games {
        assert!(game["name"].as_str().unwrap().starts_with("GS1"));
    }

    // Game setter 2 lists their games (should see only 1)
    let (list_status, list_body) = common::json_request(
        &app,
        http::Method::GET,
        "/api/game-setter/games",
        None,
        Some(&game_setter2_token),
    )
    .await;

    assert_eq!(list_status, StatusCode::OK);
    let games = list_body.as_array().unwrap();
    assert_eq!(games.len(), 1);
    assert!(games[0]["name"].as_str().unwrap().starts_with("GS2"));
}

#[tokio::test]
async fn game_setter_can_delete_own_game() {
    let app = common::setup_app(&common::unique_name("game_delete_")).await;
    let game_setter_token = common::game_setter_token(&app).await;

    // Create a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Delete the game
    let (delete_status, _) = common::json_request(
        &app,
        http::Method::DELETE,
        &format!("/api/game-setter/games/{}", game_id),
        None,
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    // Verify game is deleted
    let (get_status, _) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/games/{}", game_id),
        None,
        None,
    )
    .await;

    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn game_setter_cannot_delete_others_game() {
    let app = common::setup_app(&common::unique_name("game_delete_")).await;
    let game_setter1_token = common::game_setter_token(&app).await;
    let game_setter2_token = common::game_setter_token(&app).await;

    // Game setter 1 creates a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter1_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Game setter 2 tries to delete (should fail)
    let (delete_status, delete_body) = common::json_request(
        &app,
        http::Method::DELETE,
        &format!("/api/game-setter/games/{}", game_id),
        None,
        Some(&game_setter2_token),
    )
    .await;

    assert_eq!(delete_status, StatusCode::FORBIDDEN);
    assert!(
        delete_body["error"]
            .as_str()
            .unwrap()
            .contains("permission")
    );

    // Verify game still exists
    let (get_status, _) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/games/{}", game_id),
        None,
        None,
    )
    .await;

    assert_eq!(get_status, StatusCode::OK);
}

#[tokio::test]
async fn admin_can_delete_any_game() {
    let app = common::setup_app(&common::unique_name("game_delete_")).await;
    let game_setter_token = common::game_setter_token(&app).await;
    let admin_token = common::admin_token(&app).await;

    // Game setter creates a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Admin deletes the game (should succeed)
    let (delete_status, _) = common::json_request(
        &app,
        http::Method::DELETE,
        &format!("/api/game-setter/games/{}", game_id),
        None,
        Some(&admin_token),
    )
    .await;

    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    // Verify game is deleted
    let (get_status, _) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/games/{}", game_id),
        None,
        None,
    )
    .await;

    assert_eq!(get_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn game_code_upload_validates_code_length() {
    let app = common::setup_app(&common::unique_name("game_code_")).await;
    let game_setter_token = common::game_setter_token(&app).await;

    // Create a game
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Game {}", common::unique_name("")),
            "description": "Test game",
            "supported_languages": ["rust"]
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = common::extract_thing_id(&create_body["id"]);

    // Try to upload empty code (should fail)
    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "rust",
            "code_content": ""
        })),
        Some(&game_setter_token),
    )
    .await;

    assert_eq!(upload_status, StatusCode::BAD_REQUEST);
    assert!(
        upload_body["error"]
            .as_str()
            .unwrap()
            .contains("1 byte to 1MB")
    );
}
