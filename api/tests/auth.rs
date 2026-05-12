mod db;

use api::{
    config::Config,
    models::{LoginRequest, RegisterRequest, User, UserRole},
    services::{auth},
};

fn cfg() -> auth::AuthConfig {
    auth::AuthConfig {
        jwt_secret: "test-secret".to_string(),
        jwt_expiration: 3600,
    }
}

mod jwt_clock_skew {
    use api::models::{Claims, UserRole};
    use api::services::auth::{self, AuthConfig};
    use chrono::Utc;
    use jsonwebtoken::{EncodingKey, Header, encode};

    const SECRET: &str = "test-secret-key-min-32-bytes-please";

    fn make_token(secret: &str, exp_offset_secs: i64, iat_offset_secs: i64) -> String {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: "user:test".into(),
            email: "test@local".into(),
            role: UserRole::Player,
            exp: (now + exp_offset_secs) as usize,
            iat: (now + iat_offset_secs) as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn cfg() -> AuthConfig {
        AuthConfig {
            jwt_secret: SECRET.to_string(),
            jwt_expiration: 3600,
        }
    }

    #[test]
    fn validate_token_accepts_fresh_token() {
        let tok = make_token(SECRET, 3600, 0);
        let claims = auth::validate_token(&cfg(), &tok).unwrap();
        assert_eq!(claims.sub, "user:test");
    }

    #[test]
    fn validate_token_rejects_expired() {
        let tok = make_token(SECRET, -3600, -7200);
        assert!(
            auth::validate_token(&cfg(), &tok).is_err(),
            "expired must fail"
        );
    }

    #[test]
    fn validate_token_rejects_wrong_secret() {
        let tok = make_token("totally-different-secret-xxxxxxxx", 3600, 0);
        assert!(auth::validate_token(&cfg(), &tok).is_err());
    }

    #[test]
    fn validate_token_rejects_garbage() {
        assert!(auth::validate_token(&cfg(), "not.a.token").is_err());
        assert!(auth::validate_token(&cfg(), "").is_err());
    }
}

async fn get_bob_user(db: &api::db::Database) -> api::models::User {
    let config = Config::from_env();
    <api::db::Database as axel_core::repo::user::UserRepo>::find_by_email(db, &config.bob.email)
        .await
        .unwrap()
        .expect("Bob user should exist")
}

#[tokio::test]
async fn test_auth_service_password_hashing() {
    let password = "test_password_123";
    let hash = auth::hash_password(password).unwrap();
    assert!(auth::verify_password(password, &hash).unwrap());
    assert!(!auth::verify_password("wrong_password", &hash).unwrap());
}

#[tokio::test]
async fn test_auth_service_jwt_generation() {
    let user = User {
        id: Some(surrealdb::types::RecordId::new("user", "test123")),
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password_hash: None,
        role: UserRole::Player,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: surrealdb::types::Datetime::default(),
        updated_at: surrealdb::types::Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };
    let token = auth::generate_token(&cfg(), &user).unwrap();
    assert!(!token.is_empty());
    let claims = auth::validate_token(&cfg(), &token).unwrap();
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.role, UserRole::Player);
}

#[tokio::test]
async fn test_auth_service_invalid_token() {
    let result = auth::validate_token(&cfg(), "invalid.token.here");
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
    let token1 = auth::generate_reset_token();
    let token2 = auth::generate_reset_token();
    assert_eq!(token1.len(), 32);
    assert_eq!(token2.len(), 32);
    assert_ne!(token1, token2);
}

#[tokio::test]
async fn test_registration_and_login_flow() {
    let db = db::setup_test_db().await;
    let config = Config::from_env();

    let bob_user = get_bob_user(&db).await;
    let password = &config.bob.password;

    let password_hash = bob_user.password_hash.as_ref().unwrap();
    assert!(auth::verify_password(password, password_hash).unwrap());

    let token = auth::generate_token(&cfg(), &bob_user).unwrap();
    assert!(!token.is_empty());

    let claims = auth::validate_token(&cfg(), &token).unwrap();
    assert_eq!(claims.email, bob_user.email);
}

#[tokio::test]
async fn test_password_reset_flow() {
    let db = db::setup_test_db().await;
    let config = Config::from_env();

    let mut bob_user = get_bob_user(&db).await;
    let original_password_hash = bob_user.password_hash.clone().unwrap();

    let raw_reset_token = auth::generate_reset_token();
    let reset_token_hash = auth::hash_reset_token(&raw_reset_token);
    bob_user.password_reset_token = Some(reset_token_hash);

    let user_id = bob_user.id.clone().unwrap();
    <api::db::Database as axel_core::repo::user::UserRepo>::update(&db, user_id.clone(), bob_user)
        .await
        .unwrap();

    let updated_user = <api::db::Database as axel_core::repo::user::UserRepo>::find_by_email(&db, &config.bob.email)
        .await
        .unwrap()
        .unwrap();
    assert!(updated_user.password_reset_token.is_some());

    let new_password_hash = auth::hash_password("new_password_123").unwrap();
    let mut reset_user = updated_user;
    reset_user.password_hash = Some(new_password_hash.clone());
    reset_user.password_reset_token = None;

    <api::db::Database as axel_core::repo::user::UserRepo>::update(&db, user_id.clone(), reset_user)
        .await
        .unwrap();

    let final_user = <api::db::Database as axel_core::repo::user::UserRepo>::find_by_email(&db, &config.bob.email)
        .await
        .unwrap()
        .unwrap();
    assert!(
        auth::verify_password("new_password_123", final_user.password_hash.as_ref().unwrap())
            .unwrap()
    );

    let mut restore_user = final_user;
    restore_user.password_hash = Some(original_password_hash);
    <api::db::Database as axel_core::repo::user::UserRepo>::update(&db, user_id, restore_user).await.unwrap();
}
