use axel_tournament::{
    models::{LoginRequest, RegisterRequest, User, UserRole},
    services::AuthService,
};

#[tokio::test]
async fn test_auth_service_password_hashing() {
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let password = "test_password_123";
    let hash = auth_service.hash_password(password).unwrap();
    assert!(auth_service.verify_password(password, &hash).unwrap());
    assert!(
        !auth_service
            .verify_password("wrong_password", &hash)
            .unwrap()
    );
}

#[tokio::test]
async fn test_auth_service_jwt_generation() {
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let user = User {
        id: Some(surrealdb::sql::Thing::from(("user", "test123"))),
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password_hash: None,
        role: UserRole::Player,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: surrealdb::sql::Datetime::default(),
        updated_at: surrealdb::sql::Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };
    let token = auth_service.generate_token(&user).unwrap();
    assert!(!token.is_empty());
    let claims = auth_service.validate_token(&token).unwrap();
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.role, UserRole::Player);
}

#[tokio::test]
async fn test_auth_service_invalid_token() {
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let result = auth_service.validate_token("invalid.token.here");
    assert!(result.is_err());
}

#[test]
fn test_register_request_validation() {
    use validator::Validate;
    let valid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password: "password123".to_string(),
        location: Some("US".to_string()),
    };
    assert!(valid_request.validate().is_ok());
    let invalid_email = RegisterRequest {
        email: "not-an-email".to_string(),
        username: "testuser".to_string(),
        password: "password123".to_string(),
        location: Some("US".to_string()),
    };
    assert!(invalid_email.validate().is_err());
    let short_password = RegisterRequest {
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password: "short".to_string(),
        location: Some("US".to_string()),
    };
    assert!(short_password.validate().is_err());
    let short_username = RegisterRequest {
        email: "test@example.com".to_string(),
        username: "ab".to_string(),
        password: "password123".to_string(),
        location: Some("US".to_string()),
    };
    assert!(short_username.validate().is_err());
}

#[test]
fn test_login_request_validation() {
    use validator::Validate;
    let valid_request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };
    assert!(valid_request.validate().is_ok());
    let invalid_email = LoginRequest {
        email: "not-an-email".to_string(),
        password: "password123".to_string(),
    };
    assert!(invalid_email.validate().is_err());
}

#[tokio::test]
async fn test_reset_token_generation() {
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let token1 = auth_service.generate_reset_token();
    let token2 = auth_service.generate_reset_token();
    assert_eq!(token1.len(), 32);
    assert_eq!(token2.len(), 32);
    assert_ne!(token1, token2);
}

use axel_tournament::{
    db,
    services::{auth, user},
};

async fn setup_test_db() -> axel_tournament::db::Database {
    let config = axel_tournament::config::Config::from_env()
        .expect("Failed to load config from environment");
    db::connect(&config.database)
        .await
        .expect("Failed to connect to test database")
}

fn unique_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", prefix, timestamp)
}

#[tokio::test]
async fn test_registration_and_login_flow() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    
    let email = format!("{}@test.com", unique_name("auth_user"));
    let username = unique_name("auth_user");
    let password = "password123";
    
    // Register user
    let created_user = user::create_user(
        &db,
        email.clone(),
        username.clone(),
        Some(auth_service.hash_password(password).unwrap()),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    
    assert_eq!(created_user.email, email);
    assert_eq!(created_user.username, username);
    
    // Login flow - verify password
    let fetched_user = auth::get_user_by_email(&db, &email).await.unwrap().unwrap();
    let password_hash = fetched_user.password_hash.as_ref().unwrap();
    assert!(auth_service.verify_password(password, password_hash).unwrap());
    
    // Generate token for user
    let token = auth_service.generate_token(&fetched_user).unwrap();
    assert!(!token.is_empty());
    
    // Validate token
    let claims = auth_service.validate_token(&token).unwrap();
    assert_eq!(claims.email, email);
}

#[tokio::test]
async fn test_password_reset_flow() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    
    let email = format!("{}@test.com", unique_name("reset_user"));
    let username = unique_name("reset_user");
    
    // Create user
    let mut user = user::create_user(
        &db,
        email.clone(),
        username,
        Some(auth_service.hash_password("password123").unwrap()),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    
    // Generate reset token
    let raw_reset_token = auth_service.generate_reset_token();
    let reset_token_hash = auth_service.hash_reset_token(&raw_reset_token);
    user.password_reset_token = Some(reset_token_hash);
    
    let user_id = user.id.clone().unwrap();
    user::update_user(&db, user_id.clone(), user).await.unwrap();
    
    // Verify reset token and update password
    let updated_user = auth::get_user_by_email(&db, &email).await.unwrap().unwrap();
    assert!(updated_user.password_reset_token.is_some());
    
    // Simulate password reset
    let new_password_hash = auth_service.hash_password("new_password_123").unwrap();
    let mut reset_user = updated_user;
    reset_user.password_hash = Some(new_password_hash);
    reset_user.password_reset_token = None;
    
    user::update_user(&db, user_id, reset_user).await.unwrap();
    
    // Verify new password works
    let final_user = auth::get_user_by_email(&db, &email).await.unwrap().unwrap();
    assert!(auth_service.verify_password("new_password_123", &final_user.password_hash.unwrap()).unwrap());
}
