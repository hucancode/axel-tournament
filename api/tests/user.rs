use axel_tournament::{
    db,
    services::{auth::AuthService, user},
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
async fn test_ban_and_unban_user() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = user::create_user(
        &db,
        format!("{}@test.com", unique_name("user")),
        unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = created_user.id.unwrap();
    let banned = user::ban_user(&db, user_id.clone(), "Violation".to_string())
        .await
        .unwrap();
    assert!(banned.is_banned);
    assert_eq!(banned.ban_reason.unwrap(), "Violation");
    let unbanned = user::unban_user(&db, user_id).await.unwrap();
    assert!(!unbanned.is_banned);
    assert!(unbanned.ban_reason.is_none());
}

#[tokio::test]
async fn test_list_users() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    for _ in 0..3 {
        let password_hash = auth_service.hash_password("password123").unwrap();
        user::create_user(
            &db,
            format!("{}@test.com", unique_name("list_user")),
            unique_name("list_user"),
            Some(password_hash),
            "US".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
    }
    let users = user::list_users(&db, Some(5), Some(0)).await.unwrap();
    assert!(users.len() >= 3);
}

#[tokio::test]
async fn test_user_profile_update() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    
    // Create user
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = user::create_user(
        &db,
        format!("{}@test.com", unique_name("profile_user")),
        unique_name("profile_user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    
    let user_id = created_user.id.clone().unwrap();
    
    // Update location
    let mut updated_user = created_user;
    updated_user.location = "CA".to_string();
    
    let result = user::update_user(&db, user_id.clone(), updated_user).await.unwrap();
    assert_eq!(result.location, "CA");
    
    // Verify update persisted by checking the returned result
    assert_eq!(result.location, "CA");
}
