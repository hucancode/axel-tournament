use api::{db, config::Config};

pub async fn setup_test_db() -> api::db::Database {
    let config = Config::from_env();
    db::connect(&config.database)
        .await
        .expect("Failed to connect to test database")
}
