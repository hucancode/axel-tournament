use judge::Config;
use judge::db::Database;

/// Create a test database connection
pub async fn setup_test_db() -> Database {
    let config = Config::from_env();
    use judge::db::connect;
    connect(
        &config.database_url,
        &config.database_ns,
        &config.database_db,
        &config.database_user,
        &config.database_pass,
    ).await
    .expect("Failed to connect to test database")
}
