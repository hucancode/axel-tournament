mod common;

use axel_tournament::models::{CreateGameRequest, CreateGameTemplateRequest, ProgrammingLanguage, GameType};
use axum::http::{Method, StatusCode};
use common::{extract_thing_id, game_setter_token, json_request, setup_app, unique_name};
use serde_json::json;

fn build_game_request(
    name: String,
    description: &str,
    supported_languages: Vec<ProgrammingLanguage>,
) -> CreateGameRequest {
    let game_language = supported_languages
        .first()
        .cloned()
        .unwrap_or(ProgrammingLanguage::Rust);
    CreateGameRequest {
        name,
        description: description.to_string(),
        game_type: GameType::Automated,
        supported_languages,
        game_code: "fn main() {}".to_string(),
        game_language,
        frontend_code: None,
        rounds_per_match: 3,
        repetitions: 1,
        timeout_ms: 5000,
        cpu_limit: 1.0,
        turn_timeout_ms: 200,
        memory_limit_mb: 2,
    }
}

#[tokio::test]
async fn test_game_setter_can_create_game() {
    let app = setup_app().await;
    let token = game_setter_token(&app).await;

    let game_request = build_game_request(
        format!("Test Game {}", unique_name("")),
        "A test game",
        vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go],
    );

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
    let app = setup_app().await;

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

    let game_request = build_game_request(
        format!("Test Game {}", unique_name("")),
        "A test game",
        vec![ProgrammingLanguage::Rust],
    );

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
async fn test_create_game_template() {
    let app = setup_app().await;
    let token = game_setter_token(&app).await;

    // Create a game first
    let game_request = build_game_request(
        format!("Template Game {}", unique_name("")),
        "A game with templates",
        vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go],
    );

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
    assert!(
        template["template_code"]
            .as_str()
            .unwrap()
            .contains("Hello, world")
    );
}

#[tokio::test]
async fn test_list_game_templates() {
    let app = setup_app().await;
    let token = game_setter_token(&app).await;

    // Create a game
    let game_request = build_game_request(
        format!("Multi-Lang Game {}", unique_name("")),
        "A game with multiple templates",
        vec![ProgrammingLanguage::Rust, ProgrammingLanguage::Go],
    );

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
