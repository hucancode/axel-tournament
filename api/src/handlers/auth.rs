use crate::{
    AppState,
    error::{ApiError, ApiResult},
    models::*,
    services::{self, AuthService},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::Redirect,
};
use chrono::{Duration, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, TokenResponse, TokenUrl, basic::BasicClient,
};
use reqwest;
use rand::{rng, Rng};
use serde::Deserialize;
use validator::Validate;

type GoogleClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

const OAUTH_STATE_COOKIE: &str = "oauth_state";

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

fn build_state_cookie(state: &str, ttl_seconds: i64, secure: bool) -> Result<header::HeaderValue, ApiError> {
    let max_age = ttl_seconds.max(60);
    let mut cookie = format!(
        "{}={}; Max-Age={}; Path=/api/auth/google/callback; HttpOnly; SameSite=Lax",
        OAUTH_STATE_COOKIE,
        state,
        max_age
    );
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
        .parse()
        .map_err(|_| ApiError::Internal("Failed to build OAuth cookie".to_string()))
}

fn expire_state_cookie(secure: bool) -> Result<header::HeaderValue, ApiError> {
    let mut cookie = format!(
        "{}=; Max-Age=0; Path=/api/auth/google/callback; HttpOnly; SameSite=Lax",
        OAUTH_STATE_COOKIE
    );
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
        .parse()
        .map_err(|_| ApiError::Internal("Failed to clear OAuth cookie".to_string()))
}

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let header_value = headers.get(header::COOKIE)?.to_str().ok()?;
    header_value.split(';').find_map(|pair| {
        let mut parts = pair.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim();
        if key == name {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn normalize_username(name: &str, email: &str) -> String {
    let base = if name.trim().is_empty() {
        email.split('@').next().unwrap_or("player")
    } else {
        name
    };
    let mut normalized: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if normalized.is_empty() {
        normalized = "player".to_string();
    }
    if normalized.len() < 3 {
        normalized.push_str("player");
    }
    normalized.truncate(50);
    normalized
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
        let reset_token_hash = state.auth_service.hash_reset_token(&reset_token);
        let expires: surrealdb::sql::Datetime = (Utc::now() + Duration::hours(1)).into();
        user.password_reset_token = Some(reset_token_hash);
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
    let token = state.auth_service.hash_reset_token(&payload.token);
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
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    id: String,
    email: String,
    name: String,
    #[serde(default)]
    verified_email: Option<bool>,
}

pub async fn google_login(
    State(state): State<AppState>,
) -> Result<(HeaderMap, Redirect), ApiError> {
    let client = build_google_client(&state.config)?;
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("openid".to_string()))
        .add_scope(oauth2::Scope::new("email".to_string()))
        .add_scope(oauth2::Scope::new("profile".to_string()))
        .url();
    let mut headers = HeaderMap::new();
    let cookie = build_state_cookie(
        csrf_token.secret(),
        state.config.oauth.state_ttl_seconds,
        state.config.oauth.cookie_secure,
    )?;
    headers.insert(header::SET_COOKIE, cookie);
    Ok((headers, Redirect::to(auth_url.as_str())))
}

pub async fn google_callback(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<GoogleCallbackQuery>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Redirect), ApiError> {
    if let Some(error) = query.error.as_deref() {
        let detail = query
            .error_description
            .as_deref()
            .unwrap_or("OAuth error");
        return Err(ApiError::Auth(format!("Google OAuth error: {} ({})", error, detail)));
    }
    let code = query
        .code
        .as_deref()
        .ok_or_else(|| ApiError::Auth("Missing OAuth code".to_string()))?;
    let state_value = query
        .state
        .as_deref()
        .ok_or_else(|| ApiError::Auth("Missing OAuth state".to_string()))?;
    let cookie_state = get_cookie(&headers, OAUTH_STATE_COOKIE)
        .ok_or_else(|| ApiError::Auth("Missing OAuth state cookie".to_string()))?;
    if cookie_state != state_value {
        return Err(ApiError::Auth("Invalid OAuth state".to_string()));
    }
    let client = build_google_client(&state.config)?;
    let http_client = reqwest::Client::new();
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
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
    if user_info.email.trim().is_empty() {
        return Err(ApiError::Auth("Google account email missing".to_string()));
    }
    if user_info.verified_email != Some(true) {
        return Err(ApiError::Forbidden(
            "Google account email is not verified".to_string(),
        ));
    }
    // Check if user exists with this OAuth ID
    let user = if let Some(existing_user) =
        services::auth::get_user_by_oauth(&state.db, "google", &user_info.id).await?
    {
        existing_user
    } else if let Some(existing_user) =
        services::auth::get_user_by_email(&state.db, &user_info.email).await?
    {
        if existing_user.oauth_provider.is_some() {
            return Err(ApiError::Conflict(
                "Email already registered with another sign-in method".to_string(),
            ));
        }
        let mut updated_user = existing_user;
        updated_user.oauth_provider = Some(OAuthProvider::Google);
        updated_user.oauth_id = Some(user_info.id.clone());
        let user_id = updated_user
            .id
            .as_ref()
            .ok_or_else(|| ApiError::Internal("User ID is missing".to_string()))?
            .clone();
        services::user::update_user(&state.db, user_id, updated_user).await?
    } else {
        let base_username = normalize_username(&user_info.name, &user_info.email);
        let mut username = base_username.clone();
        let mut attempts = 0;
        while services::auth::get_user_by_username(&state.db, &username).await?.is_some() {
            if attempts >= 5 {
                return Err(ApiError::Conflict(
                    "Unable to allocate unique username".to_string(),
                ));
            }
            let suffix = rng().random_range(1000..=9999);
            username = format!("{base_username}-{suffix}");
            username.truncate(50);
            attempts += 1;
        }
        // Create new user
        services::user::create_user(
            &state.db,
            user_info.email,
            username,
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

    // Get frontend URL from config
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Redirect to frontend with just the token
    let redirect_url = format!(
        "{}/auth/google/callback?token={}",
        frontend_url,
        token
    );

    let mut response_headers = HeaderMap::new();
    let cookie = expire_state_cookie(state.config.oauth.cookie_secure)?;
    response_headers.insert(header::SET_COOKIE, cookie);

    Ok((response_headers, Redirect::to(&redirect_url)))
}
