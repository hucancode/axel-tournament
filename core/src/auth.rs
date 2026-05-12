use axum::http::HeaderMap;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use surrealdb::types::RecordId;
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::user::{User, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

pub fn encode_token(cfg: &AuthConfig, claims: &Claims) -> AppResult<String> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(AppError::from)
}

pub fn validate_token(secret: &str, token: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(AppError::from)
}

/// Pull the `Authorization: Bearer <token>` header, validate the JWT,
/// load the user row, ensure not banned. Returns the claims so the
/// caller (axum middleware in api/judge) can stash them in extensions.
///
/// Both binaries share this body so the auth contract — header shape,
/// ban check, error mapping — stays in one place.
pub async fn authenticate_bearer<C: Connection>(
    secret: &str,
    headers: &HeaderMap,
    db: &Surreal<C>,
) -> AppResult<Claims> {
    let header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Auth("Missing authorization header".into()))?;
    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Auth("Invalid authorization format".into()))?;
    let claims = validate_token(secret, token)?;
    let uid = RecordId::parse_simple(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user id".into()))?;
    let user: Option<User> = db.select(&uid).await?;
    let user = user.ok_or_else(|| AppError::Auth("User not found".into()))?;
    if user.is_banned {
        return Err(AppError::Forbidden("User is banned".into()));
    }
    Ok(claims)
}
