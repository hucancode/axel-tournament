use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: usize,   // expiration
}

/// Auth middleware that automatically intercepts protected routes
pub async fn auth_middleware<G>(
    State(state): State<Arc<crate::app_state::AppState<G>>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode>
where
    G: crate::models::game::Game + Clone + Send + Sync + 'static,
{
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = validate_jwt(token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Store claims in request extensions for handlers to access
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => Err(format!("Invalid JWT: {}", e)),
    }
}

/// Extension trait to easily get claims from request
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
