use crate::{
    AppState,
    error::ApiError,
    models::{Claims, UserRole},
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub async fn game_setter_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| ApiError::Auth("Unauthorized".to_string()))?;

    // Allow Admin or GameSetter roles
    if claims.role != UserRole::GameSetter && claims.role != UserRole::Admin {
        return Err(ApiError::Forbidden("Game setter access required".to_string()));
    }

    Ok(next.run(req).await)
}
