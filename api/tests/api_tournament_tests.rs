mod common;

use axum::http::{self, StatusCode};

#[tokio::test]
async fn tournament_create_join_and_leave() {
    let app = common::setup_app(&common::unique_name("tournament_api_")).await;
    let admin_token = common::admin_token(&app).await;
    // Create game
    let (_, game_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/games",
        Some(common::game_payload(
            format!("Tournament Game {}", common::unique_name("")),
            "For tournament tests",
        )),
        Some(&admin_token),
    )
    .await;
    let game_id = common::extract_thing_id(&game_body["id"]);
    // Create tournament
    let (t_status, tournament_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/tournaments",
        Some(serde_json::json!({
            "game_id": game_id.clone(),
            "name": format!("Tournament {}", common::unique_name("")),
            "description": "API tournament",
            "min_players": 2,
            "max_players": 16
        })),
        Some(&admin_token),
    )
    .await;
    assert!(t_status == StatusCode::CREATED || t_status == StatusCode::OK);
    let tournament_id = common::extract_thing_id(&tournament_body["id"]);
    // Register player
    let player_email = format!("{}@test.com", common::unique_name("tour_player"));
    let (_, player_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": player_email,
            "username": common::unique_name("tour_player"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;
    let player_token = player_body["token"].as_str().unwrap();
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
    let (participants_status, participants) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/tournaments/{}/participants", tournament_id),
        None,
        None,
    )
    .await;
    assert_eq!(participants_status, StatusCode::OK);
    assert_eq!(participants.as_array().unwrap().len(), 1);
    // Leave tournament
    let (leave_status, leave_body) = common::json_request(
        &app,
        http::Method::DELETE,
        &format!("/api/tournaments/{}/leave", tournament_id),
        None,
        Some(player_token),
    )
    .await;
    if !(leave_status == StatusCode::NO_CONTENT || leave_status == StatusCode::OK) {
        panic!(
            "leave tournament failed: status {} body {:?}",
            leave_status, leave_body
        );
    }
}
