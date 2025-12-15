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
