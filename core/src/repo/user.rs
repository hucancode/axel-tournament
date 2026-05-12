use async_trait::async_trait;
use surrealdb::types::{Datetime, RecordId};
use surrealdb::{Connection, Surreal};

use crate::error::{AppError, AppResult};
use crate::models::user::{OAuthProvider, User, UserRole};

pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub location: String,
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_id: Option<String>,
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create(&self, new_user: NewUser) -> AppResult<User>;
    async fn update(&self, user_id: RecordId, user: User) -> AppResult<User>;
    async fn ban(&self, user_id: RecordId, reason: String) -> AppResult<User>;
    async fn unban(&self, user_id: RecordId) -> AppResult<User>;
    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> AppResult<Vec<User>>;
    async fn get_by_id(&self, user_id: &RecordId) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_oauth(&self, provider: &str, oauth_id: &str) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
}

#[async_trait]
impl<C: Connection> UserRepo for Surreal<C> {
    async fn create(&self, new_user: NewUser) -> AppResult<User> {
        let user = User {
            id: None,
            email: new_user.email,
            username: new_user.username,
            password_hash: new_user.password_hash,
            role: UserRole::Player,
            location: new_user.location,
            oauth_provider: new_user.oauth_provider,
            oauth_id: new_user.oauth_id,
            is_banned: false,
            ban_reason: None,
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
            password_reset_token: None,
            password_reset_expires: None,
        };
        let created: Option<User> = self.create("user").content(user).await?;
        created.ok_or_else(|| AppError::Internal("Failed to create user".into()))
    }

    async fn update(&self, user_id: RecordId, user: User) -> AppResult<User> {
        let updated: Option<User> = self.update(&user_id).content(user).await?;
        updated.ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    async fn ban(&self, user_id: RecordId, reason: String) -> AppResult<User> {
        let mut result = self
            .query("UPDATE $user_id SET is_banned = true, ban_reason = $reason, updated_at = $now")
            .bind(("user_id", user_id))
            .bind(("reason", reason))
            .bind(("now", Datetime::default()))
            .await?;
        let users: Vec<User> = result.take(0)?;
        users
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    async fn unban(&self, user_id: RecordId) -> AppResult<User> {
        let mut result = self
            .query("UPDATE $user_id SET is_banned = false, ban_reason = NONE, updated_at = $now")
            .bind(("user_id", user_id))
            .bind(("now", Datetime::default()))
            .await?;
        let users: Vec<User> = result.take(0)?;
        users
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> AppResult<Vec<User>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        let mut result = self
            .query("SELECT * FROM user ORDER BY created_at DESC LIMIT $limit START $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        Ok(result.take(0)?)
    }

    async fn get_by_id(&self, user_id: &RecordId) -> AppResult<User> {
        let user: Option<User> = self.select(user_id).await?;
        user.ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let mut result = self
            .query("SELECT * FROM user WHERE email = $email")
            .bind(("email", email.to_string()))
            .await?;
        let users: Vec<User> = result.take(0)?;
        Ok(users.into_iter().next())
    }

    async fn find_by_oauth(&self, provider: &str, oauth_id: &str) -> AppResult<Option<User>> {
        let mut result = self
            .query("SELECT * FROM user WHERE oauth_provider = $provider AND oauth_id = $oauth_id")
            .bind(("provider", provider.to_string()))
            .bind(("oauth_id", oauth_id.to_string()))
            .await?;
        let users: Vec<User> = result.take(0)?;
        Ok(users.into_iter().next())
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let mut result = self
            .query("SELECT * FROM user WHERE username = $username")
            .bind(("username", username.to_string()))
            .await?;
        let users: Vec<User> = result.take(0)?;
        Ok(users.into_iter().next())
    }
}
