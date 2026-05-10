use crate::{
    error::{ApiError, ApiResult},
    models::{Claims, User},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sha2::{Digest, Sha256};
use surrealdb::types::ToSql;

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

pub fn hash_password(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| ApiError::PasswordHash)
}

pub fn verify_password(password: &str, hash: &str) -> ApiResult<bool> {
    let parsed = PasswordHash::new(hash).map_err(|_| ApiError::PasswordHash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn generate_token(cfg: &AuthConfig, user: &User) -> ApiResult<String> {
    let now = Utc::now().timestamp() as usize;
    let user_id = user
        .id
        .as_ref()
        .ok_or_else(|| ApiError::Internal("User ID is missing".to_string()))?
        .to_sql();
    let claims = Claims {
        sub: user_id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp: (now as i64 + cfg.jwt_expiration) as usize,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(ApiError::from)
}

pub fn validate_token(cfg: &AuthConfig, token: &str) -> ApiResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(ApiError::from)
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

