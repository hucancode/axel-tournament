use crate::{
    AppState,
    db::Database,
    error::{AppError, AppResult},
    models::{Claims, UserInfo},
};
use axel_core::repo::user::UserRepo;
use axum::{Extension, Json, extract::State};
use surrealdb::types::RecordId;

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserInfo>> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user id".to_string()))?;
    let u = <Database as UserRepo>::get_by_id(&state.db, &user_id).await?;
    Ok(Json(u.to_info()?))
}

pub async fn update_location(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<UserInfo>> {
    let location = payload["location"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("Location is required".to_string()))?;
    if location.len() != 2 {
        return Err(AppError::Validation(
            "Location must be a 2-letter country code".to_string(),
        ));
    }
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user id".to_string()))?;
    let mut u = <Database as UserRepo>::get_by_id(&state.db, &user_id).await?;
    u.location = location.to_uppercase();
    let user_id = u.id.as_ref().unwrap().clone();
    let updated = <Database as UserRepo>::update(&state.db, user_id, u).await?;
    Ok(Json(updated.to_info()?))
}
