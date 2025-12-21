mod common;

use axum::http::{self, StatusCode};

#[tokio::test]
async fn create_and_list_submissions() {
    let app = common::setup_app(&common::unique_name("submission_api_")).await;
    let admin_token = common::admin_token(&app).await;
    // Game
    let (_, game_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/games",
        Some(common::game_payload(
            format!("Submission Game {}", common::unique_name("")),
            "Game for submissions",
        )),
        Some(&admin_token),
    )
    .await;
    let game_id = common::extract_thing_id(&game_body["id"]);
    // Tournament
    let (_, tournament_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/tournaments",
        Some(serde_json::json!({
            "game_id": game_id.clone(),
            "name": format!("Submission Tournament {}", common::unique_name("")),
            "description": "Tournament for submissions",
            "min_players": 2,
            "max_players": 8
        })),
        Some(&admin_token),
    )
    .await;
    let tournament_id = common::extract_thing_id(&tournament_body["id"]);
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
            "tournament_id": tournament_id.clone(),
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
    let submission_thing = format!("submission:{}", submission_id);
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
        &format!("/api/submissions/{}", submission_thing),
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    let fetched_id = common::extract_thing_id(&get_body["id"]);
    assert_eq!(fetched_id, submission_thing);
}
