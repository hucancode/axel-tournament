use crate::{
    AppState,
    db::Database,
    error::AppResult,
    models::{UserInfo, rid},
};
use axel_core::repo::user::UserRepo;
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
) -> AppResult<Json<Vec<UserInfo>>> {
    let users = <Database as UserRepo>::list(&state.db, query.limit, query.offset).await?;
    let infos: Result<Vec<UserInfo>, _> = users.iter().map(|u| u.to_info()).collect();
    Ok(Json(infos?))
}

pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<BanUserRequest>,
) -> AppResult<Json<UserInfo>> {
    let u = <Database as UserRepo>::ban(&state.db, rid("user", user_id), payload.reason).await?;
    Ok(Json(u.to_info()?))
}

pub async fn unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> AppResult<Json<UserInfo>> {
    let u = <Database as UserRepo>::unban(&state.db, rid("user", user_id)).await?;
    Ok(Json(u.to_info()?))
}
