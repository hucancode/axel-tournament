use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub oauth: OAuthConfig,
    pub email: EmailConfig,
    pub app: AppConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminConfig {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64, // in seconds
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub cookie_secure: bool,
    pub state_ttl_seconds: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub max_code_size_mb: usize,
    pub default_location: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenv::dotenv().ok();
        Ok(Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| "ws://localhost:8000".to_string()),
                user: env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string()),
                pass: env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string()),
                namespace: env::var("DATABASE_NS").unwrap_or_else(|_| "axel".to_string()),
                database: env::var("DATABASE_DB").unwrap_or_else(|_| "axel".to_string()),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| "supersecret".to_string()),
                expiration: env::var("JWT_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()
                    .unwrap_or(86400),
            },
            oauth: OAuthConfig {
                google_client_id: env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| "".to_string()),
                google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
                    .unwrap_or_else(|_| "".to_string()),
                google_redirect_uri: env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:8080/api/auth/google/callback".to_string()
                }),
                cookie_secure: env::var("OAUTH_COOKIE_SECURE")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                state_ttl_seconds: env::var("OAUTH_STATE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300),
            },
            email: EmailConfig {
                smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
                smtp_port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .unwrap_or(587),
                smtp_username: env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string()),
                smtp_password: env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string()),
                from_address: env::var("EMAIL_FROM")
                    .unwrap_or_else(|_| "noreply@axel-tournament.com".to_string()),
            },
            app: AppConfig {
                max_code_size_mb: env::var("MAX_CODE_SIZE_MB")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                default_location: env::var("DEFAULT_LOCATION").unwrap_or_else(|_| "US".to_string()),
            },
            admin: AdminConfig {
                email: env::var("ADMIN_EMAIL")
                    .unwrap_or_else(|_| "admin@axel-tournament.com".to_string()),
                password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "123456".to_string()),
            },
        })
    }
}
