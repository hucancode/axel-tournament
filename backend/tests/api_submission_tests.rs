mod common;

use axum::http::{self, StatusCode};

fn extract_id(body: &serde_json::Value) -> String {
    body["id"]["id"]["String"]
        .as_str()
        .or_else(|| body["id"]["id"].as_str())
        .unwrap_or_default()
        .to_string()
}

#[tokio::test]
async fn create_and_list_submissions() {
    let app = common::setup_app(&common::unique_name("submission_api_")).await;
    let admin_token = common::admin_token(&app).await;
    // Game
    let (_, game_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/games",
        Some(serde_json::json!({
            "name": format!("Submission Game {}", common::unique_name("")),
            "description": "Game for submissions",
            "rules": {},
            "supported_languages": ["rust"]
        })),
        Some(&admin_token),
    )
    .await;
    let game_id = extract_id(&game_body);
    // Tournament
    let (_, tournament_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/tournaments",
        Some(serde_json::json!({
            "game_id": format!("game:{}", game_id),
            "name": format!("Submission Tournament {}", common::unique_name("")),
            "description": "Tournament for submissions",
            "min_players": 2,
            "max_players": 8
        })),
        Some(&admin_token),
    )
    .await;
    let tournament_id = extract_id(&tournament_body);
    // Player
    let (_, user_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": format!("{}@test.com", common::unique_name("submit_user")),
            "username": common::unique_name("submit_user"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;
    let player_token = user_body["token"].as_str().unwrap();
    // Join tournament
    let (join_status, _) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/tournaments/{}/join", tournament_id),
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(join_status, StatusCode::CREATED);
    // Create submission
    let (status, submission_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/submissions",
        Some(serde_json::json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": "fn main() { println!(\"hello\"); }"
        })),
        Some(player_token),
    )
    .await;
    if !(status == StatusCode::CREATED || status == StatusCode::OK) {
        panic!(
            "create submission failed: status {} body {:?}",
            status, submission_body
        );
    }
    let submission_id = submission_body["id"].as_str().unwrap();
    // List submissions for user
    let (list_status, list_body) = common::json_request(
        &app,
        http::Method::GET,
        "/api/submissions",
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert!(list_body.as_array().unwrap().len() >= 1);
    // Get single submission
    let (get_status, get_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/submissions/{}", submission_id),
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    let fetched_id = get_body["id"]["id"]["String"]
        .as_str()
        .or_else(|| get_body["id"]["id"].as_str())
        .unwrap_or_default();
    assert_eq!(fetched_id, submission_id);
}
