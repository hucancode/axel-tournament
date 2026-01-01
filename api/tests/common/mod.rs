#![allow(dead_code)]

use axel_tournament::{
    AppState, config::Config, db, router, services::{AuthService, EmailService}
};
use axum::{
    Router,
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
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

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn unique_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}{}_{}", prefix, timestamp, counter)
}


pub async fn setup_app() -> TestApp {
    let config = Config::from_env().expect("Failed to load config from environment");
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
    db::seed_admin_user(&db, &config.admin.email, admin_password_hash)
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

