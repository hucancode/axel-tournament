use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{OAuthProvider, User, UserRole},
};
use surrealdb::types::{Datetime, RecordId};

pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub location: String,
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_id: Option<String>,
}

pub async fn create_user(db: &Database, new_user: NewUser) -> ApiResult<User> {
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
    let created: Option<User> = db.create("user").content(user).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create user".to_string()))
}

pub async fn update_user(db: &Database, user_id: RecordId, user: User) -> ApiResult<User> {
    let updated: Option<User> = db.update(&user_id).content(user).await?;
    updated.ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn ban_user(db: &Database, user_id: RecordId, ban_reason: String) -> ApiResult<User> {
    let mut result = db
        .query("UPDATE $user_id SET is_banned = true, ban_reason = $reason, updated_at = $now")
        .bind(("user_id", user_id))
        .bind(("reason", ban_reason))
        .bind(("now", Datetime::default()))
        .await?;
    let users: Vec<User> = result.take(0)?;
    users
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn unban_user(db: &Database, user_id: RecordId) -> ApiResult<User> {
    let mut result = db
        .query("UPDATE $user_id SET is_banned = false, ban_reason = NONE, updated_at = $now")
        .bind(("user_id", user_id))
        .bind(("now", Datetime::default()))
        .await?;
    let users: Vec<User> = result.take(0)?;
    users
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn list_users(
    db: &Database,
    limit: Option<u32>,
    offset: Option<u32>,
) -> ApiResult<Vec<User>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let mut result = db
        .query("SELECT * FROM user ORDER BY created_at DESC LIMIT $limit START $offset")
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await?;
    let users: Vec<User> = result.take(0)?;
    Ok(users)
}

pub async fn get_user_by_id(db: &Database, user_id: RecordId) -> ApiResult<User> {
    let user: Option<User> = db.select(&user_id).await?;
    user.ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn get_user_by_email(db: &Database, email: &str) -> ApiResult<Option<User>> {
    let mut result = db
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", email.to_string()))
        .await?;
    let users: Vec<User> = result.take(0)?;
    Ok(users.into_iter().next())
}

pub async fn get_user_by_oauth(
    db: &Database,
    provider: &str,
    oauth_id: &str,
) -> ApiResult<Option<User>> {
    let mut result = db
        .query("SELECT * FROM user WHERE oauth_provider = $provider AND oauth_id = $oauth_id")
        .bind(("provider", provider.to_string()))
        .bind(("oauth_id", oauth_id.to_string()))
        .await?;
    let users: Vec<User> = result.take(0)?;
    Ok(users.into_iter().next())
}

pub async fn get_user_by_username(db: &Database, username: &str) -> ApiResult<Option<User>> {
    let mut result = db
        .query("SELECT * FROM user WHERE username = $username")
        .bind(("username", username.to_string()))
        .await?;
    let users: Vec<User> = result.take(0)?;
    Ok(users.into_iter().next())
}
