use crate::{
    AppState,
    error::{AppError, AppResult},
    models::{Claims, UserRole},
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims =
        axel_core::auth::authenticate_bearer(&state.auth.jwt_secret, req.headers(), &state.db)
            .await?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub async fn admin_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Auth("Unauthorized".to_string()))?;
    if claims.role != UserRole::Admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(next.run(req).await)
}

pub trait RequestExt {
    fn claims(&self) -> AppResult<&Claims>;
}

impl RequestExt for Request {
    fn claims(&self) -> AppResult<&Claims> {
        self.extensions()
            .get::<Claims>()
            .ok_or_else(|| AppError::Auth("Unauthorized".to_string()))
    }
}
