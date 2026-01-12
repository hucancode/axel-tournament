use crate::app_state::AppState;
use crate::config::Config;
use crate::games;
use crate::handlers;
use crate::handlers::room;
use crate::services::room::websocket;
use crate::middleware;
use axum::http::{header, Method};
use axum::routing::get;
use axum::{middleware as axum_middleware, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn create_router(
    config: &Config,
    tic_tac_toe_state: Arc<AppState<games::TicTacToe>>,
    rps_state: Arc<AppState<games::RockPaperScissors>>,
    pd_state: Arc<AppState<games::PrisonersDilemma>>,
) -> Router {
    tracing::info!("CORS: Allowing origin: {}", config.frontend_url);

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/capacity", get(handlers::get_capacity::<games::TicTacToe>))
        .with_state(tic_tac_toe_state.clone());

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route(
            "/api/rooms",
            get(room::list_rooms::<games::TicTacToe>).post(room::create_room::<games::TicTacToe>),
        )
        .route(
            "/api/rooms/{room_id}",
            get(room::get_room::<games::TicTacToe>),
        )
        .with_state(tic_tac_toe_state.clone())
        .route_layer(axum_middleware::from_fn_with_state(
            tic_tac_toe_state.clone(),
            middleware::auth::auth_middleware,
        ));

    // WebSocket endpoints (auth via LOGIN message, not HTTP headers)
    let websocket_routes = Router::new()
        .route(
            "/ws/tic-tac-toe/{room_id}",
            get(websocket::ws_get_room::<games::TicTacToe>),
        )
        .with_state(tic_tac_toe_state.clone())
        .route(
            "/ws/rock-paper-scissors/{room_id}",
            get(websocket::ws_get_room::<games::RockPaperScissors>),
        )
        .with_state(rps_state.clone())
        .route(
            "/ws/prisoners-dilemma/{room_id}",
            get(websocket::ws_get_room::<games::PrisonersDilemma>),
        )
        .with_state(pd_state.clone());

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(websocket_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(
                    config.frontend_url
                        .parse::<axum::http::HeaderValue>()
                        .expect("Invalid FRONTEND_URL"),
                )
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
                .allow_credentials(true),
        )
}
