use axel_tournament::{config::Config, db, handlers, middleware, services::{AuthService, EmailService}, AppState};
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn health_check() -> &'static str {
    "OK"
}

fn create_router(state: AppState) -> Router {
    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/reset-password", post(handlers::request_password_reset))
        .route("/api/auth/confirm-reset", post(handlers::confirm_password_reset))
        .route("/api/auth/google", get(handlers::google_login))
        .route("/api/auth/google/callback", get(handlers::google_callback))
        .route("/api/games", get(handlers::list_games))
        .route("/api/games/:id", get(handlers::get_game))
        .route("/api/tournaments", get(handlers::list_tournaments))
        .route("/api/tournaments/:id", get(handlers::get_tournament))
        .route("/api/tournaments/:id/participants", get(handlers::get_tournament_participants))
        .route("/api/leaderboard", get(handlers::get_leaderboard));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/users/profile", get(handlers::get_profile))
        .route("/api/users/location", patch(handlers::update_location))
        .route("/api/tournaments/:id/join", post(handlers::join_tournament))
        .route("/api/tournaments/:id/leave", delete(handlers::leave_tournament))
        .route("/api/submissions", post(handlers::create_submission))
        .route("/api/submissions", get(handlers::list_submissions))
        .route("/api/submissions/:id", get(handlers::get_submission))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Admin routes
    let admin_routes = Router::new()
        .route("/api/admin/games", post(handlers::create_game))
        .route("/api/admin/games/:id", put(handlers::update_game))
        .route("/api/admin/games/:id", delete(handlers::delete_game))
        .route("/api/admin/tournaments", post(handlers::create_tournament))
        .route("/api/admin/tournaments/:id", patch(handlers::update_tournament))
        .route("/api/admin/users", get(handlers::list_users))
        .route("/api/admin/users/:id/ban", post(handlers::ban_user))
        .route("/api/admin/users/:id/unban", post(handlers::unban_user))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::admin_middleware,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,axel_tournament=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Axel Tournament API...");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Connect to database
    let db = db::connect(&config.database).await?;
    tracing::info!("Connected to database");

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

    let app = create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
