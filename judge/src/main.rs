use judge::{
    app_state::AppState,
    config::Config,
    db, games, router,
    services::ai_match::{self, AiRegistries},
    services::capacity::CapacityTracker,
    services::room::logic::TurnWatchConfig,
    services::submission_compiler,
    services::turn_timer,
    services::room::ws::WsContext,
    services::room::logic::RoomRegistry,
    services::storage::Storage,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
    tracing::info!("Starting judge on port {}", config.server_port);
    tracing::info!(
        "Max capacity: {}, Max claim delay: {}ms",
        config.max_capacity,
        config.max_claim_delay_ms
    );

    let db = db::connect(
        &config.database_url,
        &config.database_ns,
        &config.database_db,
        &config.database_user,
        &config.database_pass,
    )
    .await?;

    let capacity = CapacityTracker::new(config.max_capacity, config.max_claim_delay_ms);

    let owner_id = std::env::var("HOSTNAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("judge-{}", uuid::Uuid::new_v4()));
    tracing::info!("judge owner_id: {}", owner_id);

    let storage = Storage::surreal(db.clone());
    let app_state = Arc::new(AppState {
        db: db.clone(),
        capacity: capacity.clone(),
        jwt_secret: config.jwt_secret.clone(),
        meta: storage.meta.clone(),
    });

    // Per-game turn-timeout from GameMetadata. The watcher attached to
    // each opened room finalises the room with the pending players
    // marked faulted when they go past the human turn budget.
    let timeout_cb = turn_timer::db_timeout_callback(db.clone());
    let rps_meta = games::find_game_by_id("rock-paper-scissors").expect("rps metadata");
    let ttt_meta = games::find_game_by_id("tic-tac-toe").expect("ttt metadata");
    let pd_meta = games::find_game_by_id("prisoners-dilemma").expect("pd metadata");
    let rps_cfg = TurnWatchConfig {
        timeout: Duration::from_millis(rps_meta.human_turn_timeout_ms),
        on_timeout: timeout_cb.clone(),
    };
    let ttt_cfg = TurnWatchConfig {
        timeout: Duration::from_millis(ttt_meta.human_turn_timeout_ms),
        on_timeout: timeout_cb.clone(),
    };
    let pd_cfg = TurnWatchConfig {
        timeout: Duration::from_millis(pd_meta.human_turn_timeout_ms),
        on_timeout: timeout_cb,
    };

    // One per-game registry shared by both transports (WebSocket for
    // humans, stdio for AI bots). AI matches do NOT need turn watchers
    // because the bot runner enforces its own turn timeout.
    let rps_registry = Arc::new(
        RoomRegistry::<games::Rps>::new(storage.clone(), owner_id.clone())
            .with_turn_watch(rps_cfg),
    );
    let ttt_registry = Arc::new(
        RoomRegistry::<games::Ttt>::new(storage.clone(), owner_id.clone())
            .with_turn_watch(ttt_cfg),
    );
    let pd_registry = Arc::new(
        RoomRegistry::<games::Pd>::new(storage.clone(), owner_id.clone())
            .with_turn_watch(pd_cfg),
    );

    let rps_ctx = Arc::new(WsContext {
        registry: rps_registry.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });
    let ttt_ctx = Arc::new(WsContext {
        registry: ttt_registry.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });
    let pd_ctx = Arc::new(WsContext {
        registry: pd_registry.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });

    // AI tournament runner uses the same registries.
    let ai_regs = AiRegistries {
        rps: rps_registry.clone(),
        ttt: ttt_registry.clone(),
        pd: pd_registry.clone(),
    };
    ai_match::spawn(db.clone(), capacity.clone(), ai_regs);

    // Compile pending submissions upfront so match runners can reuse
    // the cached binary across every match the bot plays.
    submission_compiler::spawn(db.clone());

    // Renew leases for owned rooms every 5s with 15s TTL.
    let hb_rps = rps_registry.clone();
    let hb_ttt = ttt_registry.clone();
    let hb_pd = pd_registry.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            tokio::join!(
                hb_rps.heartbeat(Duration::from_secs(15)),
                hb_ttt.heartbeat(Duration::from_secs(15)),
                hb_pd.heartbeat(Duration::from_secs(15)),
            );
        }
    });

    let playground_regs = judge::handlers::PlaygroundRegistries {
        rps: rps_registry.clone(),
        ttt: ttt_registry.clone(),
        pd: pd_registry.clone(),
    };
    let app = router::create_router(
        &config,
        app_state,
        rps_ctx,
        ttt_ctx,
        pd_ctx,
        playground_regs,
    );

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting judge server on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
