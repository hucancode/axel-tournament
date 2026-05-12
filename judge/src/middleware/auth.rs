use axel_core::error::AppError;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub use axel_core::auth::Claims;

/// Auth middleware for protected HTTP routes.
///
/// Symmetric with api: validates JWT, then ensures the user record
/// exists and is not banned. Without this check the judge would happily
/// run code submissions from users banned in the api.
pub async fn auth_middleware(
    State(state): State<Arc<crate::app_state::AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims =
        axel_core::auth::authenticate_bearer(&state.jwt_secret, req.headers(), &state.db).await?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub trait RequestExt {
    fn claims(&self) -> Result<&Claims, StatusCode>;
    fn user_id(&self) -> Result<String, StatusCode>;
}

impl RequestExt for Request {
    fn claims(&self) -> Result<&Claims, StatusCode> {
        self.extensions()
            .get::<Claims>()
            .ok_or(StatusCode::UNAUTHORIZED)
    }

    fn user_id(&self) -> Result<String, StatusCode> {
        Ok(self.claims()?.sub.clone())
    }
}
