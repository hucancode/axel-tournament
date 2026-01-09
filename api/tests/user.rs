mod db;
use api::{
    config::Config,
    services::{auth, user},
};

async fn get_bob_user(db: &api::db::Database) -> api::models::User {
    let config = Config::from_env();
    auth::get_user_by_email(db, &config.bob.email)
        .await
        .unwrap()
        .expect("Bob user should exist")
}

async fn get_alice_user(db: &api::db::Database) -> api::models::User {
    let config = Config::from_env();
    auth::get_user_by_email(db, &config.alice.email)
        .await
        .unwrap()
        .expect("Alice user should exist")
}

#[tokio::test]
async fn test_ban_and_unban_user() {
    let db = db::setup_test_db().await;
    let bob_user = get_bob_user(&db).await;
    let user_id = bob_user.id.unwrap();

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
    let db = db::setup_test_db().await;
    let users = user::list_users(&db, Some(5), Some(0)).await.unwrap();
    // Should have at least admin, bob, and alice
    assert!(users.len() >= 3);
}

#[tokio::test]
async fn test_user_profile_update() {
    let db = db::setup_test_db().await;
    let mut alice_user = get_alice_user(&db).await;
    let user_id = alice_user.id.clone().unwrap();

    // Update location
    alice_user.location = "CA".to_string();

    let result = user::update_user(&db, user_id.clone(), alice_user)
        .await
        .unwrap();
    assert_eq!(result.location, "CA");

    // Reset location back to original for other tests
    let mut reset_user = result;
    reset_user.location = "US".to_string();
    user::update_user(&db, user_id, reset_user).await.unwrap();
}
