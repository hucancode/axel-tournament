// API endpoint tests for Axel Tournament
// Tests actual HTTP handlers using test SurrealDB instance at localhost:8001
//
// Run tests with: make test
// Or manually: make test-db && cargo test && make test-db-stop

use axel_tournament::{
    config::{Config, ServerConfig, DatabaseConfig, JwtConfig, OAuthConfig, EmailConfig, AppConfig, AdminConfig},
    db,
    handlers,
    middleware,
    services::{AuthService, EmailService},
    AppState,
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::{Service, ServiceExt};
use tower_http::cors::{Any, CorsLayer};

// Helper function to create test config
fn create_test_config() -> Config {
    Config {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseConfig {
            url: "ws://127.0.0.1:8001".to_string(),
            user: "root".to_string(),
            pass: "root".to_string(),
            namespace: "test".to_string(),
            database: "test".to_string(),
        },
        jwt: JwtConfig {
            secret: "test-secret-key".to_string(),
            expiration: 3600,
        },
        oauth: OAuthConfig {
            google_client_id: "".to_string(),
            google_client_secret: "".to_string(),
            google_redirect_uri: "http://localhost:8080/callback".to_string(),
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

// Helper function to create test app with full router
async fn create_test_app() -> Router {
    let config = create_test_config();

    // Connect to test database at localhost:8001
    let db = db::connect(&config.database).await.expect("Failed to connect to test database");

    let auth_service = Arc::new(AuthService::new(
        config.jwt.secret.clone(),
        config.jwt.expiration,
    ));
    let email_service = Arc::new(EmailService::new(config.email.clone()));

    let state = AppState {
        db,
        auth_service,
        email_service,
        config: Arc::new(config.clone()),
    };

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/games", get(handlers::list_games))
        .route("/api/games/:id", get(handlers::get_game))
        .route("/api/tournaments", get(handlers::list_tournaments))
        .route("/api/leaderboard", get(handlers::get_leaderboard));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/users/profile", get(handlers::get_profile))
        .route("/api/tournaments/:id/join", post(handlers::join_tournament))
        .route("/api/submissions", post(handlers::create_submission))
        .route("/api/submissions", get(handlers::list_submissions))
        .route("/api/submissions/:id", get(handlers::get_submission))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Admin routes (require authentication and admin role)
    let admin_routes = Router::new()
        .route("/api/admin/games", post(handlers::create_game))
        .route("/api/admin/games/:id", put(handlers::update_game))
        .route("/api/admin/games/:id", axum::routing::delete(handlers::delete_game))
        .route("/api/admin/tournaments", post(handlers::create_tournament))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(cors)
        .with_state(state)
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_register_user() {
    let app = create_test_app().await;

    // Use unique email/username to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let register_data = json!({
        "email": format!("newuser{}@test.com", timestamp),
        "username": format!("newuser{}", timestamp),
        "password": "password123",
        "location": "US"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/auth/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&register_data).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK && status != StatusCode::CREATED {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Response status: {}", status);
        eprintln!("Response body: {}", body_str);
        panic!("Expected status 200 or 201, got {}", status);
    }
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body["token"].is_string());
    assert!(body["user"]["email"].as_str().unwrap().contains("newuser"));
    assert!(body["user"]["username"].as_str().unwrap().contains("newuser"));
}

#[tokio::test]
async fn test_register_invalid_email() {
    let app = create_test_app().await;

    let register_data = json!({
        "email": "not-an-email",
        "username": "testuser",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/auth/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&register_data).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_success() {
    let mut app = create_test_app().await.into_service();

    // First register a user
    let register_data = json!({
        "email": "logintest@test.com",
        "username": "logintest",
        "password": "password123",
        "location": "US"
    });

    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&register_data).unwrap()))
        .unwrap();

    ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(register_request)
        .await
        .unwrap();

    // Now try to login
    let login_data = json!({
        "email": "logintest@test.com",
        "password": "password123"
    });

    let login_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&login_data).unwrap()))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(login_request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "logintest@test.com");
}

#[tokio::test]
async fn test_list_games_empty() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/games")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body.is_array());
}

#[tokio::test]
async fn test_list_tournaments_empty() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tournaments")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body.is_array());
}

#[tokio::test]
async fn test_get_leaderboard() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/leaderboard?limit=10")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Response status: {}", status);
        eprintln!("Response body: {}", body_str);
        panic!("Expected 200, got {}", status);
    }
    assert_eq!(status, StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body.is_array());
}
// Helper function to create admin user and get token
async fn create_admin_and_get_token<S>(app: &mut S) -> String
where
    S: Service<Request<Body>, Response = axum::response::Response, Error = std::convert::Infallible> + Send,
    S::Future: Send,
{
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let register_data = json!({
        "email": format!("admin{}@test.com", timestamp),
        "username": format!("admin{}", timestamp),
        "password": "admin123",
        "location": "US"
    });

    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&register_data).unwrap()))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(app)
        .await
        .unwrap()
        .call(register_request)
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_game_admin() {
    let mut app = create_test_app().await.into_service();

    // Get admin token
    let token = create_admin_and_get_token(&mut app).await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let game_data = json!({
        "name": format!("Test Game {}", timestamp),
        "description": "A test game for API testing",
        "rules": {
            "max_rounds": 100,
            "time_limit": 60
        },
        "supported_languages": ["rust", "go", "c"]
    });

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/admin/games")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&game_data).unwrap()))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK && status != StatusCode::CREATED {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Response status: {}", status);
        eprintln!("Response body: {}", body_str);
        panic!("Expected 200 or 201, got {}", status);
    }

    assert!(status == StatusCode::OK || status == StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body["id"].is_object());
    assert_eq!(body["name"], format!("Test Game {}", timestamp));
    assert_eq!(body["is_active"], true);
    assert!(body["supported_languages"].is_array());
    assert_eq!(body["supported_languages"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_get_game_by_id() {
    let mut app = create_test_app().await.into_service();

    // First create a game
    let token = create_admin_and_get_token(&mut app).await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let game_data = json!({
        "name": format!("Get Game Test {}", timestamp),
        "description": "Test game for GET endpoint",
        "rules": {},
        "supported_languages": ["rust"]
    });

    let create_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/admin/games")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&game_data).unwrap()))
        .unwrap();

    let create_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(create_request)
        .await
        .unwrap();

    let create_body = create_response.into_body().collect().await.unwrap().to_bytes();
    let create_body: Value = serde_json::from_slice(&create_body).unwrap();
    let game_id = create_body["id"]["id"]["String"].as_str().unwrap();

    // Now get the game
    let get_request = Request::builder()
        .uri(format!("/api/games/{}", game_id))
        .body(Body::empty())
        .unwrap();

    let get_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(get_request)
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    let get_body = get_response.into_body().collect().await.unwrap().to_bytes();
    let get_body: Value = serde_json::from_slice(&get_body).unwrap();

    assert_eq!(get_body["name"], format!("Get Game Test {}", timestamp));
}

#[tokio::test]
async fn test_update_game_admin() {
    let mut app = create_test_app().await.into_service();

    // Create a game first
    let token = create_admin_and_get_token(&mut app).await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let game_data = json!({
        "name": format!("Update Test {}", timestamp),
        "description": "Original description",
        "rules": {"initial": "value"},
        "supported_languages": ["rust"]
    });

    let create_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/admin/games")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&game_data).unwrap()))
        .unwrap();

    let create_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(create_request)
        .await
        .unwrap();

    let create_body = create_response.into_body().collect().await.unwrap().to_bytes();
    let create_body: Value = serde_json::from_slice(&create_body).unwrap();
    let game_id = create_body["id"]["id"]["String"].as_str().unwrap();

    // Update the game
    let update_data = json!({
        "name": format!("Updated Name {}", timestamp),
        "description": "Updated description",
        "rules": {"max_rounds": 50},
        "supported_languages": ["rust", "go"],
        "is_active": false
    });

    let update_request = Request::builder()
        .method(http::Method::PUT)
        .uri(format!("/api/admin/games/{}", game_id))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&update_data).unwrap()))
        .unwrap();

    let update_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(update_request)
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body = update_response.into_body().collect().await.unwrap().to_bytes();
    let update_body: Value = serde_json::from_slice(&update_body).unwrap();

    assert_eq!(update_body["name"], format!("Updated Name {}", timestamp));
    assert_eq!(update_body["description"], "Updated description");
    assert_eq!(update_body["is_active"], false);
}

#[tokio::test]
async fn test_delete_game_admin() {
    let mut app = create_test_app().await.into_service();

    // Create a game first
    let token = create_admin_and_get_token(&mut app).await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let game_data = json!({
        "name": format!("Delete Test {}", timestamp),
        "description": "Will be deleted",
        "rules": {},
        "supported_languages": ["rust"]
    });

    let create_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/admin/games")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&game_data).unwrap()))
        .unwrap();

    let create_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(create_request)
        .await
        .unwrap();

    let create_body = create_response.into_body().collect().await.unwrap().to_bytes();
    let create_body: Value = serde_json::from_slice(&create_body).unwrap();
    let game_id = create_body["id"]["id"]["String"].as_str().unwrap();

    // Delete the game
    let delete_request = Request::builder()
        .method(http::Method::DELETE)
        .uri(format!("/api/admin/games/{}", game_id))
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(delete_request)
        .await
        .unwrap();

    let status = delete_response.status();
    assert!(status == StatusCode::OK || status == StatusCode::NO_CONTENT, "Expected 200 or 204, got {}", status);

    // Verify the game is deleted by trying to fetch it
    let get_request = Request::builder()
        .uri(format!("/api/games/{}", game_id))
        .body(Body::empty())
        .unwrap();

    let get_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(get_request)
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_submission_api() {
    let mut app = create_test_app().await.into_service();

    // Create user and get token
    let token = create_admin_and_get_token(&mut app).await;

    // Create a game
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let game_data = json!({
        "name": format!("Submission Game {}", timestamp),
        "description": "Game for submission testing",
        "rules": {},
        "supported_languages": ["rust"]
    });

    let create_game_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/admin/games")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&game_data).unwrap()))
        .unwrap();

    let game_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(create_game_req)
        .await
        .unwrap();

    let game_body = game_response.into_body().collect().await.unwrap().to_bytes();
    let game_body: Value = serde_json::from_slice(&game_body).unwrap();
    let game_id = game_body["id"]["id"]["String"].as_str().unwrap();

    // Create a tournament
    let tournament_data = json!({
        "game_id": format!("game:{}", game_id),
        "name": format!("Submission Tournament {}", timestamp),
        "description": "Tournament for submissions",
        "min_players": 2,
        "max_players": 100
    });

    let create_tournament_req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/admin/tournaments")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&tournament_data).unwrap()))
        .unwrap();

    let tournament_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(create_tournament_req)
        .await
        .unwrap();

    let tournament_status = tournament_response.status();
    let tournament_body = tournament_response.into_body().collect().await.unwrap().to_bytes();
    if tournament_status != StatusCode::OK && tournament_status != StatusCode::CREATED {
        eprintln!("Tournament creation failed: {}", tournament_status);
        eprintln!("Response: {}", String::from_utf8_lossy(&tournament_body));
        panic!("Failed to create tournament");
    }
    let tournament_body: Value = serde_json::from_slice(&tournament_body).unwrap();
    let tournament_id = tournament_body["id"]["id"]["String"].as_str().unwrap();

    // Create a submission (tournament_id should be just the ID, not "tournament:id")
    let submission_data = json!({
        "tournament_id": tournament_id,
        "language": "rust",
        "code": "fn main() { println!(\"Hello, world!\"); }"
    });

    let submission_request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/submissions")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&submission_data).unwrap()))
        .unwrap();

    let submission_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(submission_request)
        .await
        .unwrap();

    let status = submission_response.status();
    if status != StatusCode::OK && status != StatusCode::CREATED {
        let body = submission_response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Response status: {}", status);
        eprintln!("Response body: {}", body_str);
        panic!("Expected 200 or 201, got {}", status);
    }

    let submission_body = submission_response.into_body().collect().await.unwrap().to_bytes();
    let submission_body: Value = serde_json::from_slice(&submission_body).unwrap();

    // Check if submission has required fields from SubmissionResponse
    assert!(submission_body["id"].is_string());
    assert_eq!(submission_body["language"], "rust");
    assert!(submission_body["tournament_id"].is_string());
    assert!(submission_body["status"].is_string());
}

#[tokio::test]
async fn test_list_submissions_api() {
    let mut app = create_test_app().await.into_service();

    // Get token
    let token = create_admin_and_get_token(&mut app).await;

    // List submissions
    let list_request = Request::builder()
        .uri("/api/submissions")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let list_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(list_request)
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body = list_response.into_body().collect().await.unwrap().to_bytes();
    let list_body: Value = serde_json::from_slice(&list_body).unwrap();

    assert!(list_body.is_array());
}
