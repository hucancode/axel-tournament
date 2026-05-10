use crate::app_state::AppState;
use crate::config::Config;
use crate::games;
use crate::handlers;
use crate::services::playground::PlaygroundRegistries;
use crate::middleware::auth::auth_middleware;
use crate::services::room::ws::{handle_ws, WsContext};
use crate::services::room::logic::RoomLogic;
use axum::extract::{Path, State, ws::WebSocketUpgrade};
use axum::http::{header, Method};
use axum::middleware::from_fn_with_state;
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub async fn ws_entry<L: RoomLogic>(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(ctx): State<Arc<WsContext<L>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws(socket, room_id, ctx))
}

#[allow(clippy::too_many_arguments)]
pub fn create_router(
    config: &Config,
    app_state: Arc<AppState>,
    rps_ctx: Arc<WsContext<games::Rps>>,
    ttt_ctx: Arc<WsContext<games::Ttt>>,
    pd_ctx: Arc<WsContext<games::Pd>>,
    chess_ctx: Arc<WsContext<games::Chess>>,
    xiangqi_ctx: Arc<WsContext<games::Xiangqi>>,
    poker_ctx: Arc<WsContext<games::Poker>>,
    playground_regs: PlaygroundRegistries,
) -> Router {
    tracing::info!("CORS: Allowing origin: {}", config.frontend_url);

    let public_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/capacity", get(handlers::get_capacity))
        .route("/api/rooms", get(handlers::list_rooms))
        .with_state(app_state.clone());

    let playground_routes = Router::new()
        .route("/api/playground/start", post(handlers::playground_start))
        .layer(from_fn_with_state(app_state.clone(), auth_middleware))
        .with_state(playground_regs);

    let rps_ws = Router::new()
        .route("/ws/rock-paper-scissors/{room_id}", get(ws_entry::<games::Rps>))
        .with_state(rps_ctx);
    let ttt_ws = Router::new()
        .route("/ws/tic-tac-toe/{room_id}", get(ws_entry::<games::Ttt>))
        .with_state(ttt_ctx);
    let pd_ws = Router::new()
        .route("/ws/prisoners-dilemma/{room_id}", get(ws_entry::<games::Pd>))
        .with_state(pd_ctx);
    let chess_ws = Router::new()
        .route("/ws/chess/{room_id}", get(ws_entry::<games::Chess>))
        .with_state(chess_ctx);
    let xiangqi_ws = Router::new()
        .route("/ws/xiangqi/{room_id}", get(ws_entry::<games::Xiangqi>))
        .with_state(xiangqi_ctx);
    let poker_ws = Router::new()
        .route("/ws/poker/{room_id}", get(ws_entry::<games::Poker>))
        .with_state(poker_ctx);
    let websocket_routes = Router::new()
        .merge(rps_ws)
        .merge(ttt_ws)
        .merge(pd_ws)
        .merge(chess_ws)
        .merge(xiangqi_ws)
        .merge(poker_ws);

    Router::new()
        .merge(public_routes)
        .merge(playground_routes)
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
