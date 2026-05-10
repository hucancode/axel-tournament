use api::{
    AppState,
    config::Config,
    db, router,
    services::{auth::AuthConfig, healer::run_healer},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
    let db = db::connect(&config.database).await?;

    let auth = AuthConfig {
        jwt_secret: config.jwt.secret.clone(),
        jwt_expiration: config.jwt.expiration,
    };

    let admin_password_hash = api::services::auth::hash_password(&config.admin.password)?;
    let bob_password_hash = api::services::auth::hash_password(&config.bob.password)?;
    let alice_password_hash = api::services::auth::hash_password(&config.alice.password)?;
    db::seed_users(
        &db,
        &config.admin.email,
        admin_password_hash,
        &config.bob.email,
        bob_password_hash,
        &config.alice.email,
        alice_password_hash,
    )
    .await?;

    let healer_db = db.clone();
    tokio::spawn(async move {
        run_healer(healer_db).await;
    });

    let state = AppState {
        db,
        auth,
        config: Arc::new(config.clone()),
    };
    let app = router::create_router(state);
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting API server on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
