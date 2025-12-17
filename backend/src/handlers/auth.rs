use crate::{
    AppState,
    error::{ApiError, ApiResult},
    models::*,
    services::{self, AuthService},
};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl,
    TokenResponse, TokenUrl, basic::BasicClient,
};
use reqwest;
use serde::Deserialize;
use validator::Validate;

type GoogleClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

fn build_google_client(config: &crate::config::Config) -> Result<GoogleClient, ApiError> {
    let client = BasicClient::new(ClientId::new(config.oauth.google_client_id.clone()))
        .set_client_secret(ClientSecret::new(config.oauth.google_client_secret.clone()))
        .set_auth_uri(
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| ApiError::Auth(format!("Invalid auth url: {}", e)))?,
        )
        .set_token_uri(
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .map_err(|e| ApiError::Auth(format!("Invalid token url: {}", e)))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(config.oauth.google_redirect_uri.clone())
                .map_err(|e| ApiError::Auth(format!("Invalid redirect url: {}", e)))?,
        );
    Ok(client)
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    // Check if user already exists
    if let Some(_) = services::auth::get_user_by_email(&state.db, &payload.email).await? {
        return Err(ApiError::Conflict("Email already registered".to_string()));
    }
    // Hash password
    let password_hash = state.auth_service.hash_password(&payload.password)?;
    // Create user
    let location = payload
        .location
        .unwrap_or_else(|| state.config.app.default_location.clone());
    let user = services::user::create_user(
        &state.db,
        payload.email,
        payload.username,
        Some(password_hash),
        location,
        None,
        None,
    )
    .await?;
    // Generate token
    let token = state.auth_service.generate_token(&user)?;
    let user_info = AuthService::user_to_info(&user)?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user_info,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    // Get user by email
    let user = services::auth::get_user_by_email(&state.db, &payload.email)
        .await?
        .ok_or_else(|| ApiError::Auth("Invalid credentials".to_string()))?;
    // Verify password
    let password_hash = user
        .password_hash
        .as_ref()
        .ok_or_else(|| ApiError::Auth("Use OAuth to login".to_string()))?;
    if !state
        .auth_service
        .verify_password(&payload.password, password_hash)?
    {
        return Err(ApiError::Auth("Invalid credentials".to_string()));
    }
    // Check if banned
    if user.is_banned {
        return Err(ApiError::Forbidden(format!(
            "Account is banned. Reason: {}",
            user.ban_reason
                .unwrap_or_else(|| "No reason provided".to_string())
        )));
    }
    // Generate token
    let token = state.auth_service.generate_token(&user)?;
    let user_info = AuthService::user_to_info(&user)?;
    Ok(Json(AuthResponse {
        token,
        user: user_info,
    }))
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    // Get user by email (don't reveal if user exists)
    if let Some(mut user) = services::auth::get_user_by_email(&state.db, &payload.email).await? {
        // Generate reset token
        let reset_token = state.auth_service.generate_reset_token();
        let expires: surrealdb::sql::Datetime = (Utc::now() + Duration::hours(1)).into();
        user.password_reset_token = Some(reset_token.clone());
        user.password_reset_expires = Some(expires);
        let user_id = user.id.as_ref().unwrap().clone();
        services::user::update_user(&state.db, user_id, user).await?;
        // Send email
        state
            .email_service
            .send_password_reset(&payload.email, &reset_token)
            .await?;
    }
    // Always return success to prevent user enumeration
    Ok(Json(serde_json::json!({
        "message": "If the email exists, a password reset link has been sent"
    })))
}

pub async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<ConfirmResetPasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    payload
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    // Find user with this reset token
    let token = payload.token.clone();
    let mut result = state
        .db
        .query("SELECT * FROM user WHERE password_reset_token = $reset_token")
        .bind(("reset_token", token))
        .await?;
    let users: Vec<User> = result.take(0)?;
    let mut user = users
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::BadRequest("Invalid or expired reset token".to_string()))?;
    // Check if token expired
    if let Some(expires) = user.password_reset_expires {
        let now: surrealdb::sql::Datetime = Utc::now().into();
        if expires < now {
            return Err(ApiError::BadRequest("Reset token has expired".to_string()));
        }
    } else {
        return Err(ApiError::BadRequest("Invalid reset token".to_string()));
    }
    // Hash new password
    let password_hash = state.auth_service.hash_password(&payload.new_password)?;
    // Update user
    user.password_hash = Some(password_hash);
    user.password_reset_token = None;
    user.password_reset_expires = None;
    let user_id = user.id.as_ref().unwrap().clone();
    services::user::update_user(&state.db, user_id, user).await?;
    Ok(Json(serde_json::json!({
        "message": "Password has been reset successfully"
    })))
}

#[derive(Deserialize)]
pub struct GoogleCallbackQuery {
    code: String,
}

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    id: String,
    email: String,
    name: String,
}

pub async fn google_login(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let client = build_google_client(&state.config)?;
    let (auth_url, _csrf_token) = client
        .authorize_url(|| oauth2::CsrfToken::new("csrf_token".to_string()))
        .add_scope(oauth2::Scope::new("email".to_string()))
        .add_scope(oauth2::Scope::new("profile".to_string()))
        .url();
    Ok(Json(serde_json::json!({
        "auth_url": auth_url.to_string()
    })))
}

pub async fn google_callback(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<GoogleCallbackQuery>,
) -> ApiResult<Json<AuthResponse>> {
    let client = build_google_client(&state.config)?;
    let http_client = reqwest::Client::new();
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await
        .map_err(|e| ApiError::Auth(format!("Failed to exchange code: {}", e)))?;
    // Get user info from Google
    let client = reqwest::Client::new();
    let user_info: GoogleUserInfo = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|e| ApiError::Auth(format!("Failed to get user info: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::Auth(format!("Failed to parse user info: {}", e)))?;
    // Check if user exists with this OAuth ID
    let user = if let Some(existing_user) =
        services::auth::get_user_by_oauth(&state.db, "google", &user_info.id).await?
    {
        existing_user
    } else {
        // Create new user
        services::user::create_user(
            &state.db,
            user_info.email,
            user_info.name,
            None,
            state.config.app.default_location.clone(),
            Some(OAuthProvider::Google),
            Some(user_info.id),
        )
        .await?
    };
    // Check if banned
    if user.is_banned {
        return Err(ApiError::Forbidden(format!(
            "Account is banned. Reason: {}",
            user.ban_reason
                .unwrap_or_else(|| "No reason provided".to_string())
        )));
    }
    // Generate token
    let token = state.auth_service.generate_token(&user)?;
    let user_info = AuthService::user_to_info(&user)?;
    Ok(Json(AuthResponse {
        token,
        user: user_info,
    }))
}
