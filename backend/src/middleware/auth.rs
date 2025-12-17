use crate::{
    AppState,
    error::{ApiError, ApiResult},
    models::{Claims, UserRole},
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use surrealdb::sql::Thing;

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
    let claims = state.auth_service.validate_token(token)?;
    // Check if user is banned
    let user = crate::services::auth::get_user_by_id(
        &state.db,
        claims
            .sub
            .parse::<Thing>()
            .map_err(|_| ApiError::Auth("Invalid user id".to_string()))?,
    )
    .await?;
    if user.is_banned {
        return Err(ApiError::Forbidden("User is banned".to_string()));
    }
    // Store claims in request extensions
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

// Extension trait to easily get claims from request
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
