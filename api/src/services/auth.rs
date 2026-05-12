use crate::{
    error::{AppError, AppResult},
    models::{Claims, User},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use surrealdb::types::ToSql;

pub use axel_core::auth::AuthConfig;

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::PasswordHash)
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash).map_err(|_| AppError::PasswordHash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn generate_token(cfg: &AuthConfig, user: &User) -> AppResult<String> {
    let now = Utc::now().timestamp() as usize;
    let user_id = user
        .id
        .as_ref()
        .ok_or_else(|| AppError::Internal("User ID is missing".to_string()))?
        .to_sql();
    let claims = Claims {
        sub: user_id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp: (now as i64 + cfg.jwt_expiration) as usize,
        iat: now,
    };
    axel_core::auth::encode_token(cfg, &claims)
}

pub fn validate_token(cfg: &AuthConfig, token: &str) -> AppResult<Claims> {
    axel_core::auth::validate_token(&cfg.jwt_secret, token)
}

pub fn generate_reset_token() -> String {
    use rand::{Rng, distr::Alphanumeric};
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub fn hash_reset_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("{:x}", digest)
}

