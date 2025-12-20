mod common;

use axel_tournament::services::tournament;
use axum::http::{self, StatusCode};
use surrealdb::sql::Thing;

#[tokio::test]
async fn leaderboard_returns_scored_players() {
    let app = common::setup_app(&common::unique_name("leaderboard_api_")).await;
    let admin_token = common::admin_token(&app).await;
    // Create game
    let (_, game_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/games",
        Some(serde_json::json!({
            "name": format!("Leaderboard Game {}", common::unique_name("")),
            "description": "Game for leaderboard API",
            "supported_languages": ["rust"]
        })),
        Some(&admin_token),
    )
    .await;
    let game_id = common::extract_thing_id(&game_body["id"]);
    // Create tournament
    let (_, tournament_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/tournaments",
        Some(serde_json::json!({
            "game_id": game_id.clone(),
            "name": format!("Leaderboard Tournament {}", common::unique_name("")),
            "description": "Tournament for leaderboard API",
            "min_players": 2,
            "max_players": 8
        })),
        Some(&admin_token),
    )
    .await;
    let tournament_id = common::extract_thing_id(&tournament_body["id"]);
    // Register two players and join
    for i in 0..2 {
        let (_, register_body) = common::json_request(
            &app,
            http::Method::POST,
            "/api/auth/register",
            Some(serde_json::json!({
                "email": format!("{}@test.com", common::unique_name(&format!("lb{}", i))),
                "username": common::unique_name("lb_user"),
                "password": "password123",
                "location": "US"
            })),
            None,
        )
        .await;
        let token = register_body["token"].as_str().unwrap().to_string();
        common::json_request(
            &app,
            http::Method::POST,
            &format!("/api/tournaments/{}/join", tournament_id),
            None,
            Some(&token),
        )
        .await;
    }
    // Update scores via service helper
    let participants = tournament::get_tournament_participants(
        &app.state.db,
        tournament_id.parse::<Thing>().unwrap(),
    )
    .await
    .unwrap();
    for (idx, participant) in participants.iter().enumerate() {
        app.state
            .db
            .query("UPDATE $id SET score = $score, rank = $rank")
            .bind(("id", participant.id.clone().unwrap()))
            .bind(("score", 100.0 - (idx as f64 * 10.0)))
            .bind(("rank", (idx + 1) as i32))
            .await
            .unwrap();
    }
    let (status, body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/leaderboard?tournament_id={}", tournament_id),
        None,
        None,
    )
    .await;
    if status != StatusCode::OK {
        panic!("leaderboard failed: status {} body {:?}", status, body);
    }
    assert!(body.as_array().unwrap().len() >= 2);
    assert!(body[0]["score"].as_f64().unwrap() >= body[1]["score"].as_f64().unwrap());
}
