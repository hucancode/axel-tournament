mod common;

use axum::http::{Method, StatusCode};
use axel_tournament::models::{
    CreateGameRequest, CreateGameTemplateRequest, CreateMatchPolicyRequest, ProgrammingLanguage,
    UploadDockerfileRequest,
};
use common::{admin_token, extract_thing_id, game_setter_token, json_request, setup_app, unique_name};
use serde_json::json;

#[tokio::test]
async fn test_game_setter_can_create_game() {
    let app = setup_app("test_game_setter_create").await;
    let token = game_setter_token(&app).await;

    let game_request = CreateGameRequest {
        name: format!("Test Game {}", unique_name("")),
        description: "A test game".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (status, game) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/games",
        Some(game_json),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(game["name"].as_str().unwrap().starts_with("Test Game"));
    assert!(game["owner_id"].is_string()); // owner_id is returned as string "user:id"
}

#[tokio::test]
async fn test_player_cannot_create_game() {
    let app = setup_app("test_player_no_game").await;

    // Register a regular player
    let unique_email = format!("player{}@test.com", unique_name(""));
    let unique_username = format!("player{}", unique_name(""));

    let (_, auth_response) = json_request(
        &app,
        Method::POST,
        "/api/auth/register",
        Some(json!({
            "email": unique_email,
            "username": unique_username,
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;

    let player_token = auth_response["token"].as_str().unwrap();

    let game_request = CreateGameRequest {
        name: format!("Test Game {}", unique_name("")),
        description: "A test game".to_string(),
        supported_languages: vec![ProgrammingLanguage::Rust],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (status, _) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/games",
        Some(game_json),
        Some(player_token),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_game_setter_can_upload_dockerfile() {
    let app = setup_app("test_upload_dockerfile").await;
    let token = game_setter_token(&app).await;

    // First create a game
    let game_request = CreateGameRequest {
        name: format!("Docker Game {}", unique_name("")),
        description: "A game with dockerfile".to_string(),
        
        supported_languages: vec![ProgrammingLanguage::Rust],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (_, game) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/games",
        Some(game_json),
        Some(&token),
    )
    .await;

    let game_id = extract_thing_id(&game["id"]);

    // Upload dockerfile
    let dockerfile_request = UploadDockerfileRequest {
        dockerfile_content: "FROM rust:1.92-slim\nCMD [\"echo\", \"hello\"]".to_string(),
    };

    let dockerfile_json = serde_json::to_value(&dockerfile_request).unwrap();
    let (status, result) = json_request(
        &app,
        Method::POST,
        &format!("/api/game-setter/games/{}/dockerfile", game_id),
        Some(dockerfile_json),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(result["message"].as_str().unwrap().contains("successfully"));
    assert_eq!(result["message"], "Dockerfile uploaded successfully");
}

#[tokio::test]
async fn test_admin_can_upload_to_any_game() {
    let app = setup_app("test_admin_dockerfile").await;
    let setter_token = game_setter_token(&app).await;
    let admin_tok = admin_token(&app).await;

    // Create game with game setter
    let game_request = CreateGameRequest {
        name: format!("Private Game {}", unique_name("")),
        description: "A private game".to_string(),
        
        supported_languages: vec![ProgrammingLanguage::Rust],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (_, game) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/games",
        Some(game_json),
        Some(&setter_token),
    )
    .await;

    let game_id = extract_thing_id(&game["id"]);

    // Admin uploads dockerfile to game setter's game
    let dockerfile_request = UploadDockerfileRequest {
        dockerfile_content: "FROM rust:1.92-slim\nCMD [\"echo\", \"hello\"]".to_string(),
    };

    let dockerfile_json = serde_json::to_value(&dockerfile_request).unwrap();
    let (status, _) = json_request(
        &app,
        Method::POST,
        &format!("/api/game-setter/games/{}/dockerfile", game_id),
        Some(dockerfile_json),
        Some(&admin_tok),
    )
    .await;

    assert_eq!(status, StatusCode::OK); // Admin should succeed
}

#[tokio::test]
async fn test_create_game_template() {
    let app = setup_app("test_game_template").await;
    let token = game_setter_token(&app).await;

    // Create a game first
    let game_request = CreateGameRequest {
        name: format!("Template Game {}", unique_name("")),
        description: "A game with templates".to_string(),
        
        supported_languages: vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (_, game) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/games",
        Some(game_json),
        Some(&token),
    )
    .await;

    let game_id = extract_thing_id(&game["id"]);

    // Create template for Rust
    let template_request = CreateGameTemplateRequest {
        game_id: game_id.clone(),
        language: "rust".to_string(),
        template_code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
    };

    let template_json = serde_json::to_value(&template_request).unwrap();
    let (status, template) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/templates",
        Some(template_json),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(template["language"], "rust");
    assert!(template["template_code"]
        .as_str()
        .unwrap()
        .contains("Hello, world"));
}

#[tokio::test]
async fn test_list_game_templates() {
    let app = setup_app("test_list_templates").await;
    let token = game_setter_token(&app).await;

    // Create a game
    let game_request = CreateGameRequest {
        name: format!("Multi-Lang Game {}", unique_name("")),
        description: "A game with multiple templates".to_string(),
        
        supported_languages: vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (_, game) = json_request(
        &app,
        Method::POST,
        "/api/game-setter/games",
        Some(game_json),
        Some(&token),
    )
    .await;

    let game_id = extract_thing_id(&game["id"]);

    // Create templates for Rust and Go
    for lang in &["rust", "go"] {
        let template_request = CreateGameTemplateRequest {
            game_id: game_id.clone(),
            language: lang.to_string(),
            template_code: format!("// {} template", lang),
        };

        let template_json = serde_json::to_value(&template_request).unwrap();
        json_request(
            &app,
            Method::POST,
            "/api/game-setter/templates",
            Some(template_json),
            Some(&token),
        )
        .await;
    }

    // List templates
    let (status, templates) = json_request(
        &app,
        Method::GET,
        &format!("/api/game-setter/templates?game_id={}", game_id),
        None,
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let templates_array = templates.as_array().unwrap();
    assert_eq!(templates_array.len(), 2);
}

#[tokio::test]
async fn test_create_match_policy() {
    let app = setup_app("test_match_policy").await;
    let admin_tok = admin_token(&app).await;

    // Create a game and tournament first
    let game_request = CreateGameRequest {
        name: format!("Policy Game {}", unique_name("")),
        description: "A game for policy testing".to_string(),
        
        supported_languages: vec![ProgrammingLanguage::Rust],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (_, game) = json_request(
        &app,
        Method::POST,
        "/api/admin/games",
        Some(game_json),
        Some(&admin_tok),
    )
    .await;

    let game_id = extract_thing_id(&game["id"]);

    // Create tournament
    let (_, tournament) = json_request(
        &app,
        Method::POST,
        "/api/admin/tournaments",
        Some(json!({
            "game_id": game_id,
            "name": "Policy Tournament",
            "description": "A tournament with custom policy",
            "status": "registration",
            "min_players": 2,
            "max_players": 10
        })),
        Some(&admin_tok),
    )
    .await;

    let tournament_id = extract_thing_id(&tournament["id"]);

    // Create match policy
    let policy_request = CreateMatchPolicyRequest {
        tournament_id: tournament_id.to_string(),
        rounds_per_match: Some(3),
        repetitions: Some(2),
        timeout_seconds: Some(120),
        cpu_limit: Some("2.0".to_string()),
        memory_limit: Some("1024m".to_string()),
        scoring_weights: Some(json!({"win": 3, "draw": 1, "loss": 0})),
    };

    let policy_json = serde_json::to_value(&policy_request).unwrap();
    let (status, policy) = json_request(
        &app,
        Method::POST,
        &format!("/api/game-setter/tournaments/{}/policy", tournament_id),
        Some(policy_json),
        Some(&admin_tok),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(policy["rounds_per_match"], 3);
    assert_eq!(policy["repetitions"], 2);
    assert_eq!(policy["timeout_seconds"], 120);
}

#[tokio::test]
async fn test_get_match_policy() {
    let app = setup_app("test_get_policy").await;
    let admin_tok = admin_token(&app).await;

    // Create game and tournament
    let game_request = CreateGameRequest {
        name: format!("Get Policy Game {}", unique_name("")),
        description: "Test".to_string(),
        
        supported_languages: vec![ProgrammingLanguage::Rust],
    };

    let game_json = serde_json::to_value(&game_request).unwrap();
    let (_, game) = json_request(
        &app,
        Method::POST,
        "/api/admin/games",
        Some(game_json),
        Some(&admin_tok),
    )
    .await;

    let game_id = extract_thing_id(&game["id"]);

    let (_, tournament) = json_request(
        &app,
        Method::POST,
        "/api/admin/tournaments",
        Some(json!({
            "game_id": game_id,
            "name": "Get Policy Tournament",
            "description": "Test",
            "status": "registration",
            "min_players": 2,
            "max_players": 10
        })),
        Some(&admin_tok),
    )
    .await;

    let tournament_id = extract_thing_id(&tournament["id"]);

    // Create policy
    let policy_request = CreateMatchPolicyRequest {
        tournament_id: tournament_id.to_string(),
        rounds_per_match: Some(5),
        repetitions: Some(1),
        timeout_seconds: Some(300),
        cpu_limit: None,
        memory_limit: None,
        scoring_weights: None,
    };

    let policy_json = serde_json::to_value(&policy_request).unwrap();
    json_request(
        &app,
        Method::POST,
        &format!("/api/game-setter/tournaments/{}/policy", tournament_id),
        Some(policy_json),
        Some(&admin_tok),
    )
    .await;

    // Get policy
    let (status, policy) = json_request(
        &app,
        Method::GET,
        &format!("/api/game-setter/tournaments/{}/policy", tournament_id),
        None,
        Some(&admin_tok),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(policy["rounds_per_match"], 5);
}
