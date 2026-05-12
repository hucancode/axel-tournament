use crate::config::DatabaseConfig;
use crate::models::{User, UserRole};
use axel_core::db::DbConnectConfig;
pub use axel_core::db::Database;
use axel_core::error::AppResult;
use surrealdb::types::Datetime;
use tracing::info;

pub async fn connect(config: &DatabaseConfig) -> AppResult<Database> {
    axel_core::db::connect(DbConnectConfig {
        url: &config.url,
        user: &config.user,
        pass: &config.pass,
        namespace: &config.namespace,
        database: &config.database,
    })
    .await
}

/// Pre-hashed credentials for the three dev seed accounts (admin / alice /
/// bob). Built once at startup so the healer can re-seed cheaply when the
/// user table is wiped.
#[derive(Clone)]
pub struct SeedCreds {
    pub admin_email: String,
    pub admin_password_hash: String,
    pub alice_email: String,
    pub alice_password_hash: String,
    pub bob_email: String,
    pub bob_password_hash: String,
}

/// Recreate admin/alice/bob when the user table is empty. Idempotent —
/// safe to invoke on every healer tick.
pub async fn seed_users(db: &Database, creds: &SeedCreds) -> Result<(), surrealdb::Error> {
    let existing: Vec<User> = db.query("SELECT * FROM user LIMIT 1").await?.take(0)?;
    if !existing.is_empty() {
        return Ok(());
    }

    let admin = User {
        id: None,
        email: creds.admin_email.clone(),
        username: "admin".to_string(),
        password_hash: Some(creds.admin_password_hash.clone()),
        role: UserRole::Admin,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };
    let _: Option<User> = db.create(("user", "admin")).content(admin).await?;
    info!("Created admin user");

    let alice = User {
        id: None,
        email: creds.alice_email.clone(),
        username: "alice".to_string(),
        password_hash: Some(creds.alice_password_hash.clone()),
        role: UserRole::Player,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };
    let _: Option<User> = db.create(("user", "alice")).content(alice).await?;
    info!("Created alice user");

    let bob = User {
        id: None,
        email: creds.bob_email.clone(),
        username: "bob".to_string(),
        password_hash: Some(creds.bob_password_hash.clone()),
        role: UserRole::Player,
        location: "US".to_string(),
        oauth_provider: None,
        oauth_id: None,
        is_banned: false,
        ban_reason: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        password_reset_token: None,
        password_reset_expires: None,
    };
    let _: Option<User> = db.create(("user", "bob")).content(bob).await?;
    info!("Created bob user");

    Ok(())
}
