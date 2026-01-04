use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub database_ns: String,
    pub database_db: String,
    pub database_user: String,
    pub database_pass: String,
    pub max_capacity: usize,
    pub max_claim_delay_ms: u64,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "localhost:8000".to_string()),
            database_ns: env::var("DATABASE_NS")
                .unwrap_or_else(|_| "tournament".to_string()),
            database_db: env::var("DATABASE_DB")
                .unwrap_or_else(|_| "axel".to_string()),
            database_user: env::var("DATABASE_USER")
                .unwrap_or_else(|_| "root".to_string()),
            database_pass: env::var("DATABASE_PASS")
                .unwrap_or_else(|_| "root".to_string()),
            max_capacity: env::var("MAX_CAPACITY")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            max_claim_delay_ms: env::var("MAX_CLAIM_DELAY_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "supersecret".to_string()),
        })
    }
}
