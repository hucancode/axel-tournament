use crate::{
    AppState,
    error::{ApiError, ApiResult},
    models::{Claims, UserInfo},
    services::user,
};
use axum::{Extension, Json, extract::State};
use surrealdb::types::RecordId;

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<UserInfo>> {
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| ApiError::Auth("Invalid user id".to_string()))?;
    let u = user::get_user_by_id(&state.db, user_id).await?;
    Ok(Json(u.to_info()?))
}

pub async fn update_location(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> ApiResult<Json<UserInfo>> {
    let location = payload["location"]
        .as_str()
        .ok_or_else(|| ApiError::BadRequest("Location is required".to_string()))?;
    if location.len() != 2 {
        return Err(ApiError::Validation(
            "Location must be a 2-letter country code".to_string(),
        ));
    }
    let user_id = RecordId::parse_simple(&claims.sub)
        .map_err(|_| ApiError::Auth("Invalid user id".to_string()))?;
    let mut u = user::get_user_by_id(&state.db, user_id).await?;
    u.location = location.to_uppercase();
    let user_id = u.id.as_ref().unwrap().clone();
    let updated = user::update_user(&state.db, user_id, u).await?;
    Ok(Json(updated.to_info()?))
}
