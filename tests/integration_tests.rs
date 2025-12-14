use axel_tournament::{services::AuthService, models::*};

#[tokio::test]
async fn test_auth_service_password_hashing() {
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let password = "test_password_123";
    let hash = auth_service.hash_password(password).unwrap();
    assert!(auth_service.verify_password(password, &hash).unwrap());
    assert!(!auth_service.verify_password("wrong_password", &hash).unwrap());
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
    // Validate token
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

#[tokio::test]
async fn test_register_request_validation() {
    use validator::Validate;
    // Valid request
    let valid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password: "password123".to_string(),
        location: Some("US".to_string()),
    };
    assert!(valid_request.validate().is_ok());
    // Invalid email
    let invalid_email = RegisterRequest {
        email: "not-an-email".to_string(),
        username: "testuser".to_string(),
        password: "password123".to_string(),
        location: Some("US".to_string()),
    };
    assert!(invalid_email.validate().is_err());
    // Short password
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

#[tokio::test]
async fn test_login_request_validation() {
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
async fn test_create_game_request_validation() {
    use validator::Validate;
    let valid_request = CreateGameRequest {
        name: "Test Game".to_string(),
        description: "A test game".to_string(),
        rules: serde_json::json!({"max_rounds": 100}),
        supported_languages: vec![ProgrammingLanguage::Rust],
    };
    assert!(valid_request.validate().is_ok());
    let empty_name = CreateGameRequest {
        name: "".to_string(),
        description: "A test game".to_string(),
        rules: serde_json::json!({}),
        supported_languages: vec![ProgrammingLanguage::Rust],
    };
    assert!(empty_name.validate().is_err());
}

#[tokio::test]
async fn test_create_tournament_request_validation() {
    use validator::Validate;
    let valid_request = CreateTournamentRequest {
        game_id: "game123".to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 2,
        max_players: 100,
        start_time: None,
        end_time: None,
    };
    assert!(valid_request.validate().is_ok());
    let low_min = CreateTournamentRequest {
        game_id: "game123".to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 1,
        max_players: 100,
        start_time: None,
        end_time: None,
    };
    assert!(low_min.validate().is_err());
    let high_max = CreateTournamentRequest {
        game_id: "game123".to_string(),
        name: "Test Tournament".to_string(),
        description: "A test tournament".to_string(),
        min_players: 2,
        max_players: 1000,
        start_time: None,
        end_time: None,
    };
    assert!(high_max.validate().is_err());
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
async fn test_tournament_status_serialization() {
    let statuses = vec![
        TournamentStatus::Scheduled,
        TournamentStatus::Registration,
        TournamentStatus::Running,
        TournamentStatus::Completed,
        TournamentStatus::Cancelled,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: TournamentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
