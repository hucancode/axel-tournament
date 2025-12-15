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
async fn list_games_public() {
    let app = common::setup_app(&common::unique_name("game_api_")).await;
    let (status, body) =
        common::json_request(&app, http::Method::GET, "/api/games", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn admin_can_crud_games() {
    let app = common::setup_app(&common::unique_name("game_api_")).await;
    let admin_token = common::admin_token(&app).await;
    let game_name = format!("Game {}", common::unique_name(""));
    let (create_status, create_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/games",
        Some(serde_json::json!({
            "name": game_name,
            "description": "API created game",
            "rules": {},
            "supported_languages": ["rust"]
        })),
        Some(&admin_token),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let game_id = extract_id(&create_body);
    let (get_status, get_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/games/{}", game_id),
        None,
        None,
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_body["name"], create_body["name"]);
    let (update_status, update_body) = common::json_request(
        &app,
        http::Method::PUT,
        &format!("/api/admin/games/{}", game_id),
        Some(serde_json::json!({
            "name": format!("{} v2", create_body["name"].as_str().unwrap()),
            "description": "Updated description",
            "rules": {"max_rounds": 10},
            "supported_languages": ["rust", "go"],
            "is_active": false
        })),
        Some(&admin_token),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(update_body["description"], "Updated description");
    assert_eq!(update_body["is_active"], false);
    let (delete_status, _) = common::json_request(
        &app,
        http::Method::DELETE,
        &format!("/api/admin/games/{}", game_id),
        None,
        Some(&admin_token),
    )
    .await;
    assert!(delete_status == StatusCode::NO_CONTENT || delete_status == StatusCode::OK);
    let (missing_status, _) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/games/{}", game_id),
        None,
        None,
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}
