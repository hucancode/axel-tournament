use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Middleware to authenticate match runner requests
/// Match runners must provide X-Match-Runner-Key header matching MATCH_RUNNER_API_KEY env var
/// If MATCH_RUNNER_API_KEY is not set, authentication is disabled (dev mode)
pub async fn match_runner_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Get the API key from environment (optional in development)
    let expected_key = match std::env::var("MATCH_RUNNER_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            // No key configured - allow access (development mode)
            eprintln!("WARNING: MATCH_RUNNER_API_KEY not set - match runner authentication disabled");
            return Ok(next.run(request).await);
        }
    };

    // Get the key from request header
    let provided_key = request
        .headers()
        .get("X-Match-Runner-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing X-Match-Runner-Key header".to_string(),
            )
        })?;

    // Validate the key
    if provided_key != expected_key {
        return Err((
            StatusCode::FORBIDDEN,
            "Invalid match runner API key".to_string(),
        ));
    }

    Ok(next.run(request).await)
}
