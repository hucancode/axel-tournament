use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{OAuthProvider, User, UserRole},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_user(
    db: &Database,
    email: String,
    username: String,
    password_hash: Option<String>,
    location: String,
    oauth_provider: Option<OAuthProvider>,
    oauth_id: Option<String>,
) -> ApiResult<User> {
    let user = User {
        id: None,
        email,
        username,
        password_hash,
        role: UserRole::Player,
        location,
        oauth_provider,
        oauth_id,
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

pub async fn update_user(db: &Database, user_id: &str, user: User) -> ApiResult<User> {
    let updated: Option<User> = db.update(("user", user_id)).content(user).await?;

    updated.ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn ban_user(
    db: &Database,
    user_id: &str,
    ban_reason: String,
) -> ApiResult<User> {
    let mut result = db
        .query("UPDATE $user_id SET is_banned = true, ban_reason = $reason, updated_at = $now")
        .bind(("user_id", Thing::from(("user", user_id))))
        .bind(("reason", ban_reason))
        .bind(("now", Datetime::default()))
        .await?;

    let users: Vec<User> = result.take(0)?;
    users
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn unban_user(db: &Database, user_id: &str) -> ApiResult<User> {
    let mut result = db
        .query("UPDATE $user_id SET is_banned = false, ban_reason = NONE, updated_at = $now")
        .bind(("user_id", Thing::from(("user", user_id))))
        .bind(("now", Datetime::default()))
        .await?;

    let users: Vec<User> = result.take(0)?;
    users
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn list_users(db: &Database, limit: Option<u32>, offset: Option<u32>) -> ApiResult<Vec<User>> {
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
