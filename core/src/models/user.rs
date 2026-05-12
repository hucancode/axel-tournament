use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct User {
    pub id: Option<RecordId>,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub role: UserRole,
    pub location: String,
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
    pub fn to_info(&self) -> crate::error::AppResult<UserInfo> {
        let id = crate::ids::bare_key(
            self.id
                .as_ref()
                .ok_or_else(|| crate::error::AppError::Internal("User ID is missing".into()))?,
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
