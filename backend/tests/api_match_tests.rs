mod common;

use axum::http::{self, StatusCode};

fn extract_id(body: &serde_json::Value) -> String {
    body["id"]["id"]["String"]
        .as_str()
        .or_else(|| body["id"]["id"].as_str())
        .unwrap_or_default()
        .to_string()
}

async fn bootstrap_game_and_tournament(
    app: &common::TestApp,
    admin_token: &str,
) -> (String, String) {
    let (_, game_body) = common::json_request(
        app,
        http::Method::POST,
        "/api/admin/games",
        Some(serde_json::json!({
            "name": format!("Match Game {}", common::unique_name("")),
            "description": "Game for match API",
            "rules": {},
            "supported_languages": ["rust"]
        })),
        Some(admin_token),
    )
    .await;
    let game_id = extract_id(&game_body);
    let (_, tournament_body) = common::json_request(
        app,
        http::Method::POST,
        "/api/admin/tournaments",
        Some(serde_json::json!({
            "game_id": format!("game:{}", game_id),
            "name": format!("Match Tournament {}", common::unique_name("")),
            "description": "Tournament for match API",
            "min_players": 2,
            "max_players": 8
        })),
        Some(admin_token),
    )
    .await;
    let tournament_id = extract_id(&tournament_body);
    (game_id, tournament_id)
}

async fn create_player_submission(
    app: &common::TestApp,
    tournament_id: &str,
    token: &str,
    join: bool,
) -> String {
    if join {
        let (join_status, _) = common::json_request(
            app,
            http::Method::POST,
            &format!("/api/tournaments/{}/join", tournament_id),
            None,
            Some(token),
        )
        .await;
        assert_eq!(join_status, StatusCode::CREATED);
    }
    let (status, body) = common::json_request(
        app,
        http::Method::POST,
        "/api/submissions",
        Some(serde_json::json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": "fn main() { println!(\"ok\"); }"
        })),
        Some(token),
    )
    .await;
    assert!(status == StatusCode::CREATED || status == StatusCode::OK);
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn admin_can_create_and_update_match() {
    let app = common::setup_app(&common::unique_name("match_api_")).await;
    let admin_token = common::admin_token(&app).await;
    let (game_id, tournament_id) = bootstrap_game_and_tournament(&app, &admin_token).await;
    // Register two players
    let player_one = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": format!("{}@test.com", common::unique_name("match_player")),
            "username": common::unique_name("match_player1"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await
    .1;
    let player_two = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": format!("{}@test.com", common::unique_name("match_player")),
            "username": common::unique_name("match_player2"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await
    .1;
    let sub_one = create_player_submission(
        &app,
        &tournament_id,
        player_one["token"].as_str().unwrap(),
        true,
    )
    .await;
    let sub_two = create_player_submission(
        &app,
        &tournament_id,
        player_two["token"].as_str().unwrap(),
        true,
    )
    .await;
    // Create match
    let (create_status, match_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/matches",
        Some(serde_json::json!({
            "tournament_id": tournament_id,
            "game_id": game_id,
            "participant_submission_ids": [sub_one, sub_two]
        })),
        Some(&admin_token),
    )
    .await;
    if create_status != StatusCode::CREATED {
        panic!(
            "create match failed: status {} body {:?}",
            create_status, match_body
        );
    }
    let match_id = extract_id(&match_body);
    // Update match result
    let (update_status, updated_match) = common::json_request(
        &app,
        http::Method::PUT,
        &format!("/api/admin/matches/{}/result", match_id),
        Some(serde_json::json!({
            "status": "completed",
            "participants": [
                {
                    "submission_id": updated_submission_id(&match_body, 0),
                    "score": 100.0,
                    "rank": 1,
                    "is_winner": true,
                    "metadata": null
                },
                {
                    "submission_id": updated_submission_id(&match_body, 1),
                    "score": 50.0,
                    "rank": 2,
                    "is_winner": false,
                    "metadata": null
                }
            ],
            "metadata": null,
            "started_at": null,
            "completed_at": null
        })),
        Some(&admin_token),
    )
    .await;
    if update_status != StatusCode::OK {
        panic!(
            "update match failed: status {} body {:?}",
            update_status, updated_match
        );
    }
    assert_eq!(updated_match["status"], "completed");
}

fn updated_submission_id(body: &serde_json::Value, index: usize) -> String {
    body["participants"][index]["submission_id"]["id"]["String"]
        .as_str()
        .or_else(|| body["participants"][index]["submission_id"]["id"].as_str())
        .unwrap()
        .to_string()
}
