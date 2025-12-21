#![allow(dead_code)]

use axel_tournament::{
    AppState,
    config::{
        AdminConfig, AppConfig, Config, DatabaseConfig, EmailConfig, JwtConfig, OAuthConfig,
        ServerConfig,
    },
    db, router,
    services::{AuthService, EmailService},
};
use axum::{
    Router,
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tower::ServiceExt;

#[derive(Clone)]
pub struct TestApp {
    pub router: Router,
    pub state: AppState,
}

pub fn extract_thing_id(value: &Value) -> String {
    // First try to parse "table:id"
    if let Some(id_str) = value.as_str() {
        return id_str.to_string();
    }
    // SurrealDB Thing IDs are serialized as: {"tb": "table", "id": {"String": "id"}}
    // We need to extract it as "table:id" format
    if let Some(tb) = value["tb"].as_str() {
        if let Some(id_str) = value["id"]["String"].as_str() {
            return format!("{}:{}", tb, id_str);
        }
    }
    panic!("Invalid Thing ID format: {:?}", value);
}

pub fn unique_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", prefix, timestamp)
}

pub fn game_payload(name: String, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "supported_languages": ["rust"],
        "game_code": "fn main() {}",
        "game_language": "rust",
        "rounds_per_match": 3,
        "repetitions": 1,
        "timeout_seconds": 120,
        "cpu_limit": "1.0",
        "memory_limit": "512m"
    })
}

fn test_config(namespace: &str) -> Config {
    Config {
        server: ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
        },
        database: DatabaseConfig {
            url: "ws://localhost:8000".to_string(),
            user: "root".to_string(),
            pass: "root".to_string(),
            namespace: namespace.to_string(),
            database: namespace.to_string(),
        },
        jwt: JwtConfig {
            secret: "test-secret-key".to_string(),
            expiration: 3600,
        },
        oauth: OAuthConfig {
            google_client_id: "".to_string(),
            google_client_secret: "".to_string(),
            google_redirect_uri: "http://localhost:8080/callback".to_string(),
            cookie_secure: false,
            state_ttl_seconds: 300,
        },
        email: EmailConfig {
            smtp_host: "smtp.test.com".to_string(),
            smtp_port: 587,
            smtp_username: "test@test.com".to_string(),
            smtp_password: "password".to_string(),
            from_address: "noreply@test.com".to_string(),
        },
        app: AppConfig {
            max_code_size_mb: 10,
            default_location: "US".to_string(),
        },
        admin: AdminConfig {
            email: "admin@test.com".to_string(),
            password: "admin123".to_string(),
        },
    }
}

pub async fn setup_app(namespace: &str) -> TestApp {
    let config = test_config(namespace);
    let db = db::connect(&config.database)
        .await
        .expect("Failed to connect to test database");
    let auth_service = Arc::new(AuthService::new(
        config.jwt.secret.clone(),
        config.jwt.expiration,
    ));
    let email_service = Arc::new(EmailService::new(config.email.clone()));
    // Seed admin user for admin-only endpoints
    let admin_password_hash = auth_service
        .hash_password(&config.admin.password)
        .expect("Failed to hash admin password");
    db::create_admin_user(&db, &config.admin.email, admin_password_hash)
        .await
        .expect("Failed to seed admin user");
    let state = AppState {
        db,
        auth_service,
        email_service,
        config: Arc::new(config),
    };
    let router = router::create_router(state.clone());
    TestApp { router, state }
}

pub async fn json_request(
    app: &TestApp,
    method: http::Method,
    uri: &str,
    payload: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let router = app.router.clone();
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header(http::header::AUTHORIZATION, format!("Bearer {}", t));
    }
    if payload.is_some() {
        builder = builder.header(http::header::CONTENT_TYPE, "application/json");
    }
    let body = payload
        .map(|p| Body::from(serde_json::to_vec(&p).unwrap()))
        .unwrap_or_else(Body::empty);
    let request = builder.body(body).unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        return (status, json!({}));
    }
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&bytes) }));
    (status, value)
}

pub async fn admin_token(app: &TestApp) -> String {
    let (_, body) = json_request(
        app,
        http::Method::POST,
        "/api/auth/login",
        Some(json!({
            "email": app.state.config.admin.email,
            "password": app.state.config.admin.password,
        })),
        None,
    )
    .await;
    body["token"].as_str().unwrap_or_default().to_string()
}

pub async fn game_setter_token(app: &TestApp) -> String {
    // Register a game setter user with unique email
    let unique_email = format!("gamesetter{}@test.com", unique_name(""));
    let unique_username = format!("gamesetter{}", unique_name(""));

    let (_, body) = json_request(
        app,
        http::Method::POST,
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

    let user_id_str = body["user"]["id"].as_str().unwrap();
    let user_id: surrealdb::sql::Thing = user_id_str.parse().unwrap();

    // Update user role to gamesetter via direct DB access
    // Note: In production, this would be done by admin
    let _: Result<Option<serde_json::Value>, _> = app
        .state
        .db
        .query("UPDATE $user_id SET role = 'gamesetter'")
        .bind(("user_id", user_id))
        .await
        .and_then(|mut r| r.take(0));

    // Login again to get token with updated role
    let (_, login_body) = json_request(
        app,
        http::Method::POST,
        "/api/auth/login",
        Some(json!({
            "email": unique_email,
            "password": "password123",
        })),
        None,
    )
    .await;

    login_body["token"].as_str().unwrap_or_default().to_string()
}
