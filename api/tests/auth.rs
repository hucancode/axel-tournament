mod db;

use api::{
    config::Config,
    models::{LoginRequest, RegisterRequest, User, UserRole},
    services::{AuthService, auth, user},
};

async fn get_bob_user(db: &api::db::Database) -> api::models::User {
    let config = Config::from_env();
    auth::get_user_by_email(db, &config.bob.email)
        .await
        .unwrap()
        .expect("Bob user should exist")
}

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

#[tokio::test]
async fn test_registration_and_login_flow() {
    let db = db::setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let config = Config::from_env();

    // Use Bob for login flow test
    let bob_user = get_bob_user(&db).await;
    let password = &config.bob.password;

    // Login flow - verify password
    let password_hash = bob_user.password_hash.as_ref().unwrap();
    assert!(
        auth_service
            .verify_password(password, password_hash)
            .unwrap()
    );

    // Generate token for user
    let token = auth_service.generate_token(&bob_user).unwrap();
    assert!(!token.is_empty());

    // Validate token
    let claims = auth_service.validate_token(&token).unwrap();
    assert_eq!(claims.email, bob_user.email);
}

#[tokio::test]
async fn test_password_reset_flow() {
    let db = db::setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let config = Config::from_env();

    // Use Bob for password reset test
    let mut bob_user = get_bob_user(&db).await;
    let original_password_hash = bob_user.password_hash.clone().unwrap();

    // Generate reset token
    let raw_reset_token = auth_service.generate_reset_token();
    let reset_token_hash = auth_service.hash_reset_token(&raw_reset_token);
    bob_user.password_reset_token = Some(reset_token_hash);

    let user_id = bob_user.id.clone().unwrap();
    user::update_user(&db, user_id.clone(), bob_user).await.unwrap();

    // Verify reset token and update password
    let updated_user = auth::get_user_by_email(&db, &config.bob.email).await.unwrap().unwrap();
    assert!(updated_user.password_reset_token.is_some());

    // Simulate password reset
    let new_password_hash = auth_service.hash_password("new_password_123").unwrap();
    let mut reset_user = updated_user;
    reset_user.password_hash = Some(new_password_hash.clone());
    reset_user.password_reset_token = None;

    user::update_user(&db, user_id.clone(), reset_user).await.unwrap();

    // Verify new password works
    let final_user = auth::get_user_by_email(&db, &config.bob.email).await.unwrap().unwrap();
    assert!(
        auth_service
            .verify_password("new_password_123", &final_user.password_hash.as_ref().unwrap())
            .unwrap()
    );

    // Reset password back to original for other tests
    let mut restore_user = final_user;
    restore_user.password_hash = Some(original_password_hash);
    user::update_user(&db, user_id, restore_user).await.unwrap();
}
