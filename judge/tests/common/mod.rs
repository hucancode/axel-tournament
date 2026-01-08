use judge::db::Database;
use judge::room::GameContext;
use surrealdb::sql::Thing;

/// Create a test database connection
pub async fn setup_test_db() -> Database {
    judge::db::connect(
        &std::env::var("DATABASE_URL").unwrap_or_else(|_| "localhost:8000".to_string()),
        &std::env::var("DATABASE_NS").unwrap_or_else(|_| "tournament".to_string()),
        &std::env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string()),
        &std::env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string()),
        &std::env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string()),
    )
    .await
    .expect("Failed to connect to test database")
}

/// Create a test GameContext
pub async fn setup_test_game_context() -> GameContext {
    let db = setup_test_db().await;
    let match_id: Thing = "match:test".parse().unwrap();
    GameContext::new(match_id, db)
}
