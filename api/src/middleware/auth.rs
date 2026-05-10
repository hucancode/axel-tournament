use crate::{
    AppState,
    error::{ApiError, ApiResult},
    models::{Claims, UserRole},
    services::{auth, user},
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use surrealdb::types::RecordId;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Auth("Missing authorization header".to_string()))?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Auth("Invalid authorization format".to_string()))?;
    let claims = auth::validate_token(&state.auth, token)?;
    let uid = RecordId::parse_simple(&claims.sub)
        .map_err(|_| ApiError::Auth("Invalid user id".to_string()))?;
    let user = user::get_user_by_id(&state.db, uid).await?;
    if user.is_banned {
        return Err(ApiError::Forbidden("User is banned".to_string()));
    }
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub async fn admin_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| ApiError::Auth("Unauthorized".to_string()))?;
    if claims.role != UserRole::Admin {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }
    Ok(next.run(req).await)
}

pub trait RequestExt {
    fn claims(&self) -> ApiResult<&Claims>;
}

impl RequestExt for Request {
    fn claims(&self) -> ApiResult<&Claims> {
        self.extensions()
            .get::<Claims>()
            .ok_or_else(|| ApiError::Auth("Unauthorized".to_string()))
    }
}
