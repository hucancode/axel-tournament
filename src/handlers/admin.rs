use crate::{
    error::ApiResult,
    models::{User, UserInfo},
    services::{self, AuthService},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
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
    let users = services::user::list_users(&state.db, query.limit, query.offset).await?;

    let user_infos: Result<Vec<UserInfo>, _> = users
        .iter()
        .map(|u| AuthService::user_to_info(u))
        .collect();

    Ok(Json(user_infos?))
}

pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<BanUserRequest>,
) -> ApiResult<Json<UserInfo>> {
    let user = services::user::ban_user(&state.db, &user_id, payload.reason).await?;
    let user_info = AuthService::user_to_info(&user)?;
    Ok(Json(user_info))
}

pub async fn unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<UserInfo>> {
    let user = services::user::unban_user(&state.db, &user_id).await?;
    let user_info = AuthService::user_to_info(&user)?;
    Ok(Json(user_info))
}
