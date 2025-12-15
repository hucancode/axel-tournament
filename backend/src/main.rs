use axel_tournament::{
    config::Config,
    db,
    router,
    services::{AuthService, EmailService},
    AppState,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_env()?;


    // Connect to database
    let db = db::connect(&config.database).await?;


    // Initialize services
    let auth_service = Arc::new(AuthService::new(
        config.jwt.secret.clone(),
        config.jwt.expiration,
    ));
    let email_service = Arc::new(EmailService::new(config.email.clone()));

    // Create seed admin user if user table is empty
    let admin_password_hash = auth_service.hash_password(&config.admin.password)?;
    db::create_admin_user(&db, &config.admin.email, admin_password_hash).await?;

    let state = AppState {
        db,
        auth_service,
        email_service,
        config: Arc::new(config.clone()),
    };

    let app = router::create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);


    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
