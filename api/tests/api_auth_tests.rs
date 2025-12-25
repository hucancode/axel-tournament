mod common;

use axel_tournament::services::{auth, user};
use axum::http::{self, StatusCode};

#[tokio::test]
async fn health_check_works() {
    let app = common::setup_app().await;
    let (status, body) = common::json_request(&app, http::Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["raw"], "OK");
}

#[tokio::test]
async fn register_and_login_flow() {
    let app = common::setup_app().await;
    let email = format!("{}@test.com", common::unique_name("auth_user"));
    let username = common::unique_name("auth_user");
    let password = "password123";
    let (status, register_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": email,
            "username": username,
            "password": password,
            "location": "US"
        })),
        None,
    )
    .await;
    assert!(status == StatusCode::CREATED || status == StatusCode::OK);
    assert!(register_body["token"].is_string());
    let (login_status, login_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/login",
        Some(serde_json::json!({
            "email": register_body["user"]["email"].as_str().unwrap(),
            "password": password
        })),
        None,
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
    assert!(login_body["token"].is_string());
}

// #[tokio::test]
async fn password_reset_round_trip() {
    let app = common::setup_app().await;
    let email = format!("{}@test.com", common::unique_name("reset_user"));
    let username = common::unique_name("reset_user");
    common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(serde_json::json!({
            "email": email,
            "username": username,
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;
    let (status, reset_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/reset-password",
        Some(serde_json::json!({ "email": email })),
        None,
    )
    .await;
    if status != StatusCode::OK {
        panic!(
            "reset-password failed: status {} body {:?}",
            status, reset_body
        );
    }
    // Fetch token from DB and confirm reset
    let mut user = auth::get_user_by_email(&app.state.db, &email)
        .await
        .unwrap()
        .unwrap();
    assert!(user.password_reset_token.is_some());
    let raw_reset_token = common::unique_name("reset_token_");
    let reset_token_hash = app.state.auth_service.hash_reset_token(&raw_reset_token);
    user.password_reset_token = Some(reset_token_hash);
    let user_id = user.id.clone().expect("user id should exist");
    user::update_user(&app.state.db, user_id, user)
        .await
        .expect("Failed to update reset token");
    let (confirm_status, confirm_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/confirm-reset",
        Some(serde_json::json!({
            "token": raw_reset_token,
            "new_password": "new_password_123"
        })),
        None,
    )
    .await;
    if confirm_status != StatusCode::OK {
        panic!(
            "confirm-reset failed: status {} body {:?}",
            confirm_status, confirm_body
        );
    }
    let (login_status, _) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/login",
        Some(serde_json::json!({
            "email": email,
            "password": "new_password_123"
        })),
        None,
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
}
