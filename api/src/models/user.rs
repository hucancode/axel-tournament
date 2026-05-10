use crate::error::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct User {
    pub id: Option<RecordId>,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>, // None for OAuth users
    pub role: UserRole,
    pub location: String, // ISO country code (e.g., "US")
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_id: Option<String>,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub password_reset_token: Option<String>,
    pub password_reset_expires: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum UserRole {
    Admin,
    Player,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum OAuthProvider {
    Google,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(equal = 2, message = "Location must be a 2-letter country code"))]
    pub location: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub username: String,
    pub role: UserRole,
    pub location: String,
    pub is_banned: bool,
}

impl User {
    pub fn to_info(&self) -> ApiResult<UserInfo> {
        let id = super::bare_key(
            self.id
                .as_ref()
                .ok_or_else(|| ApiError::Internal("User ID is missing".to_string()))?,
        );
        Ok(UserInfo {
            id,
            email: self.email.clone(),
            username: self.username.clone(),
            role: self.role.clone(),
            location: self.location.clone(),
            is_banned: self.is_banned,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: UserRole,
    pub exp: usize,
    pub iat: usize,
}
