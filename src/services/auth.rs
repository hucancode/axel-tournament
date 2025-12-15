use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Claims, User, UserInfo},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub struct AuthService {
    jwt_secret: String,
    jwt_expiration: i64,
}

impl AuthService {
    pub fn new(jwt_secret: String, jwt_expiration: i64) -> Self {
        Self {
            jwt_secret,
            jwt_expiration,
        }
    }

    pub fn hash_password(&self, password: &str) -> ApiResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| ApiError::PasswordHash)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> ApiResult<bool> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| ApiError::PasswordHash)?;
        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn generate_token(&self, user: &User) -> ApiResult<String> {
        let now = Utc::now().timestamp() as usize;
        let user_id = user
            .id
            .as_ref()
            .ok_or_else(|| ApiError::Internal("User ID is missing".to_string()))?
            .to_string();

        let claims = Claims {
            sub: user_id,
            email: user.email.clone(),
            role: user.role.clone(),
            exp: (now as i64 + self.jwt_expiration) as usize,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(ApiError::from)
    }

    pub fn validate_token(&self, token: &str) -> ApiResult<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(ApiError::from)
    }

    pub fn generate_reset_token(&self) -> String {
        use rand::{distr::Alphanumeric, Rng};
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub fn user_to_info(user: &User) -> ApiResult<UserInfo> {
        let id = user
            .id
            .as_ref()
            .ok_or_else(|| ApiError::Internal("User ID is missing".to_string()))?
            .to_string();

        Ok(UserInfo {
            id,
            email: user.email.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            location: user.location.clone(),
            is_banned: user.is_banned,
        })
    }
}

pub async fn get_user_by_id(db: &Database, user_id: &str) -> ApiResult<User> {
    // Strip "user:" prefix if present (from JWT claims which store full Thing)
    let id_only = user_id.strip_prefix("user:").unwrap_or(user_id);
    let user: Option<User> = db.select(("user", id_only)).await?;
    user.ok_or_else(|| ApiError::NotFound("User not found".to_string()))
}

pub async fn get_user_by_email(db: &Database, email: &str) -> ApiResult<Option<User>> {
    let email_owned = email.to_string();
    let mut result = db
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", email_owned))
        .await?;

    let users: Vec<User> = result.take(0)?;
    Ok(users.into_iter().next())
}

pub async fn get_user_by_oauth(
    db: &Database,
    provider: &str,
    oauth_id: &str,
) -> ApiResult<Option<User>> {
    let provider_owned = provider.to_string();
    let oauth_id_owned = oauth_id.to_string();

    let mut result = db
        .query("SELECT * FROM user WHERE oauth_provider = $provider AND oauth_id = $oauth_id")
        .bind(("provider", provider_owned))
        .bind(("oauth_id", oauth_id_owned))
        .await?;

    let users: Vec<User> = result.take(0)?;
    Ok(users.into_iter().next())
}
