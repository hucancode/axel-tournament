use crate::{AppState, handlers, middleware};
use axum::{
    Router, middleware as axum_middleware,
    routing::{delete, get, patch, post, put},
    http::{header, Method},
};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    // Build CORS layer - restrict to specific origins in production
    // For development, you can use "http://localhost:3000" or configure via environment
    let cors = CorsLayer::new()
        .allow_origin(
            std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .parse::<axum::http::HeaderValue>()
                .expect("Invalid FRONTEND_URL"),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-match-runner-key"),
        ])
        .allow_credentials(true);
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route(
            "/api/auth/reset-password",
            post(handlers::request_password_reset),
        )
        .route(
            "/api/auth/confirm-reset",
            post(handlers::confirm_password_reset),
        )
        .route("/api/auth/google", get(handlers::google_login))
        .route("/api/auth/google/callback", get(handlers::google_callback))
        .route("/api/games", get(handlers::list_games))
        .route("/api/games/{id}", get(handlers::get_game))
        .route("/api/tournaments", get(handlers::list_tournaments))
        .route("/api/tournaments/{id}", get(handlers::get_tournament))
        .route(
            "/api/tournaments/{id}/participants",
            get(handlers::get_tournament_participants),
        )
        .route("/api/matches", get(handlers::list_matches))
        .route("/api/matches/{id}", get(handlers::get_match))
        .route("/api/leaderboard", get(handlers::get_leaderboard));
    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/users/profile", get(handlers::get_profile))
        .route("/api/users/location", patch(handlers::update_location))
        .route(
            "/api/tournaments/{id}/join",
            post(handlers::join_tournament),
        )
        .route(
            "/api/tournaments/{id}/leave",
            delete(handlers::leave_tournament),
        )
        .route("/api/submissions", post(handlers::create_submission))
        .route("/api/submissions", get(handlers::list_submissions))
        .route("/api/submissions/{id}", get(handlers::get_submission))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));
    // Admin routes
    let admin_routes = Router::new()
        .route("/api/admin/games", post(handlers::create_game))
        .route("/api/admin/games/{id}", put(handlers::update_game))
        .route("/api/admin/games/{id}", delete(handlers::delete_game))
        .route("/api/admin/tournaments", post(handlers::create_tournament))
        .route(
            "/api/admin/tournaments/{id}",
            patch(handlers::update_tournament),
        )
        .route(
            "/api/admin/tournaments/{id}/start",
            post(handlers::start_tournament),
        )
        .route("/api/admin/users", get(handlers::list_users))
        .route("/api/admin/users/{id}/ban", post(handlers::ban_user))
        .route("/api/admin/users/{id}/unban", post(handlers::unban_user))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::admin_middleware,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Match runner routes (for external match management module)
    // Requires X-Match-Runner-Key header for authentication
    let match_runner_routes = Router::new()
        .route(
            "/api/matches/{id}/result",
            put(handlers::update_match_result),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::match_runner_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(match_runner_routes)
        .layer(cors)
        .with_state(state)
}
