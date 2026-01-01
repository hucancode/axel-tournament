mod common;

use axum::http::{self, StatusCode};

#[tokio::test]
async fn list_games_public() {
    let app = common::setup_app().await;
    let (status, body) =
        common::json_request(&app, http::Method::GET, "/api/games", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());

    // Verify hardcoded games are present
    let games = body.as_array().unwrap();
    let game_ids: Vec<&str> = games
        .iter()
        .filter_map(|g| g["id"].as_str())
        .collect();

    // Check that hardcoded games exist
    assert!(game_ids.contains(&"rock-paper-scissors"), "Should have rock-paper-scissors game");
    assert!(game_ids.contains(&"prisoners-dilemma"), "Should have prisoners-dilemma game");
    assert!(game_ids.contains(&"tic-tac-toe"), "Should have tic-tac-toe game");
}

#[tokio::test]
async fn get_game_by_id() {
    let app = common::setup_app().await;
    let (status, body) = common::json_request(
        &app,
        http::Method::GET,
        "/api/games/rock-paper-scissors",
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], "rock-paper-scissors");
    assert_eq!(body["name"], "Rock Paper Scissors");
}
