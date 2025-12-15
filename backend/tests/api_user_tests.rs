mod common;

use axel_tournament::services::auth;
use axum::http::{self, StatusCode};

#[tokio::test]
async fn profile_update_and_admin_ban_flow() {
    let app = common::setup_app(&common::unique_name("user_api_")).await;
    let admin_token = common::admin_token(&app).await;

    // Register player
    let email = format!("{}@test.com", common::unique_name("user_api"));
    let (_, register_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": email,
            "username": common::unique_name("user_api"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;
    let player_token = register_body["token"].as_str().unwrap();

    // Profile
    let (profile_status, _profile_body) = common::json_request(
        &app,
        http::Method::GET,
        "/api/users/profile",
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(profile_status, StatusCode::OK);

    // Update location
    let (update_status, update_body) = common::json_request(
        &app,
        http::Method::PATCH,
        "/api/users/location",
        Some(serde_json::json!({ "location": "CA" })),
        Some(player_token),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(update_body["location"], "CA");

    // Ban user
    let user = auth::get_user_by_email(&app.state.db, &email)
        .await
        .unwrap()
        .unwrap();
    let user_id = user.id.unwrap().id.to_string();

    let (ban_status, banned_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/admin/users/{}/ban", user_id),
        Some(serde_json::json!({ "reason": "test ban" })),
        Some(&admin_token),
    )
    .await;
    assert_eq!(ban_status, StatusCode::OK);
    assert!(banned_body["is_banned"].as_bool().unwrap());

    // Access should now be forbidden
    let (forbidden_status, _) = common::json_request(
        &app,
        http::Method::GET,
        "/api/users/profile",
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN);

    // Unban
    let (unban_status, unban_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/admin/users/{}/unban", user_id),
        None,
        Some(&admin_token),
    )
    .await;
    assert_eq!(unban_status, StatusCode::OK);
    assert!(!unban_body["is_banned"].as_bool().unwrap());

    let (profile_status_after, _) = common::json_request(
        &app,
        http::Method::GET,
        "/api/users/profile",
        None,
        Some(player_token),
    )
    .await;
    assert_eq!(profile_status_after, StatusCode::OK);

    // List users
    let (list_status, list_body) = common::json_request(
        &app,
        http::Method::GET,
        "/api/admin/users",
        None,
        Some(&admin_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert!(list_body.as_array().unwrap().len() >= 1);
}
