use crate::{AppState, handlers, middleware};
use axum::{
    Router,
    http::{Method, header},
    middleware as axum_middleware,
    routing::{delete, get, patch, post, put},
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
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
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

    let match_runner_routes = Router::new()
        .route(
            "/api/matches/{id}/result",
            put(handlers::update_match_result),
        );

    // Game Setter routes (require GameSetter or Admin role)
    let game_setter_routes = Router::new()
        // Game management
        .route("/api/game-setter/games", get(handlers::list_my_games))
        .route(
            "/api/game-setter/games",
            post(handlers::create_game_as_game_setter),
        )
        .route("/api/game-setter/games/{id}", put(handlers::update_game))
        .route(
            "/api/game-setter/games/{id}",
            delete(handlers::delete_game_as_game_setter),
        )
        .route(
            "/api/game-setter/games/{id}/dockerfile",
            post(handlers::upload_dockerfile),
        )
        .route(
            "/api/game-setter/games/{id}/game-code",
            post(handlers::upload_game_code),
        )
        // Template management
        .route(
            "/api/game-setter/templates",
            post(handlers::create_template),
        )
        .route(
            "/api/game-setter/templates/{game_id}/{language}",
            get(handlers::get_template),
        )
        .route(
            "/api/game-setter/templates/{game_id}/{language}",
            put(handlers::update_template),
        )
        .route("/api/game-setter/templates", get(handlers::list_templates))
        // Tournament management (game setters can create tournaments for their games)
        .route(
            "/api/game-setter/tournaments",
            post(handlers::create_tournament),
        )
        .route(
            "/api/game-setter/tournaments/{id}",
            patch(handlers::update_tournament),
        )
        .route(
            "/api/game-setter/tournaments/{id}/start",
            post(handlers::start_tournament),
        )
        // Match policy
        .route(
            "/api/game-setter/tournaments/{id}/policy",
            post(handlers::create_policy),
        )
        .route(
            "/api/game-setter/tournaments/{id}/policy",
            get(handlers::get_policy),
        )
        .route(
            "/api/game-setter/tournaments/{id}/policy",
            put(handlers::update_policy),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::game_setter_middleware,
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
        .merge(game_setter_routes)
        .merge(match_runner_routes)
        .layer(cors)
        .with_state(state)
}
