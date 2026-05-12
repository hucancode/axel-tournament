use api::{
    AppState,
    config::Config,
    db::{self, SeedCreds},
    router,
    services::{
        auth::AuthConfig,
        healer::{bootstrap, run_bootstrap_loop, run_healer},
    },
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

    // Seed credentials are always built and handed to the bootstrap
    // loop so dropping the user table at runtime self-heals. Operators
    // who don't want auto-seed override admin/alice/bob env vars (or
    // set SEED_USERS=false to disable entirely).
    let seed = if std::env::var("SEED_USERS").ok().as_deref() == Some("false") {
        None
    } else {
        let creds = SeedCreds {
            admin_email: config.admin.email.clone(),
            admin_password_hash: api::services::auth::hash_password(&config.admin.password)?,
            alice_email: config.alice.email.clone(),
            alice_password_hash: api::services::auth::hash_password(&config.alice.password)?,
            bob_email: config.bob.email.clone(),
            bob_password_hash: api::services::auth::hash_password(&config.bob.password)?,
        };
        // Run a synchronous bootstrap once at startup so the API never
        // serves traffic before schema + dev users exist. The recurring
        // loop below keeps it that way if anything wipes the table later.
        bootstrap(&db, Some(&creds)).await?;
        Some(Arc::new(creds))
    };

    let mut supervisor = axel_core::tasks::Supervisor::new();
    let healer_db = db.clone();
    supervisor.spawn_forever("healer", move || {
        let db = healer_db.clone();
        async move { run_healer(db).await }
    });
    let bootstrap_db = db.clone();
    let bootstrap_seed = seed.clone();
    supervisor.spawn_forever("bootstrap", move || {
        let db = bootstrap_db.clone();
        let seed = bootstrap_seed.clone();
        async move { run_bootstrap_loop(db, seed).await }
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
