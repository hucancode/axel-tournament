use crate::{
    AppState,
    error::ApiResult,
    models::{Claims, UserInfo},
    services::{self, AuthService},
};
use axum::{Extension, Json, extract::State};

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<UserInfo>> {
    let user = services::auth::get_user_by_id(&state.db, &claims.sub).await?;
    let user_info = AuthService::user_to_info(&user)?;
    Ok(Json(user_info))
}

pub async fn update_location(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> ApiResult<Json<UserInfo>> {
    let location = payload["location"]
        .as_str()
        .ok_or_else(|| crate::error::ApiError::BadRequest("Location is required".to_string()))?;
    if location.len() != 2 {
        return Err(crate::error::ApiError::Validation(
            "Location must be a 2-letter country code".to_string(),
        ));
    }
    let mut user = services::auth::get_user_by_id(&state.db, &claims.sub).await?;
    user.location = location.to_uppercase();
    let user_id = user.id.as_ref().unwrap().id.to_string();
    let updated_user = services::user::update_user(&state.db, &user_id, user).await?;
    let user_info = AuthService::user_to_info(&updated_user)?;
    Ok(Json(user_info))
}
