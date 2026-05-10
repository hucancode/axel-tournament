use crate::{
    AppState,
    error::ApiResult,
    models::{UserInfo, rid},
    services::user,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BanUserRequest {
    reason: String,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> ApiResult<Json<Vec<UserInfo>>> {
    let users = user::list_users(&state.db, query.limit, query.offset).await?;
    let infos: Result<Vec<UserInfo>, _> = users.iter().map(|u| u.to_info()).collect();
    Ok(Json(infos?))
}

pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<BanUserRequest>,
) -> ApiResult<Json<UserInfo>> {
    let u = user::ban_user(&state.db, rid("user", user_id), payload.reason).await?;
    Ok(Json(u.to_info()?))
}

pub async fn unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<UserInfo>> {
    let u = user::unban_user(&state.db, rid("user", user_id)).await?;
    Ok(Json(u.to_info()?))
}
