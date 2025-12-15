mod common;

use axel_tournament::{
    config::DatabaseConfig,
    db,
    services::{auth::AuthService, user},
};

async fn setup_test_db() -> axel_tournament::db::Database {
    let namespace = common::unique_name("test_user_");
    let config = DatabaseConfig {
        url: "ws://127.0.0.1:8001".to_string(),
        user: "root".to_string(),
        pass: "root".to_string(),
        namespace: namespace.clone(),
        database: namespace,
    };
    db::connect(&config)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_ban_and_unban_user() {
    let db = setup_test_db().await;
    let auth_service = AuthService::new("test-secret".to_string(), 3600);
    let password_hash = auth_service.hash_password("password123").unwrap();
    let created_user = user::create_user(
        &db,
        format!("{}@test.com", common::unique_name("user")),
        common::unique_name("user"),
        Some(password_hash),
        "US".to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    let user_id = created_user.id.unwrap().id.to_string();
    let banned = user::ban_user(&db, &user_id, "Violation".to_string())
        .await
        .unwrap();
    assert!(banned.is_banned);
    assert_eq!(banned.ban_reason.unwrap(), "Violation");
    let unbanned = user::unban_user(&db, &user_id).await.unwrap();
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
            format!("{}@test.com", common::unique_name("list_user")),
            common::unique_name("list_user"),
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
