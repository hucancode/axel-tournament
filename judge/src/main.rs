use judge::{
    app_state::AppState,
    config::Config,
    db, games, router,
    services::ai_match::{self, AiRegistries},
    services::capacity::CapacityTracker,
    services::room::ws::WsContext,
    services::room_logic::RoomRegistry,
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

    // One per-game registry shared by both transports (WebSocket for
    // humans, stdio for AI bots).
    let rps_registry = Arc::new(RoomRegistry::<games::Rps>::new(storage.clone(), owner_id.clone()));
    let ttt_registry = Arc::new(RoomRegistry::<games::Ttt>::new(storage.clone(), owner_id.clone()));
    let pd_registry = Arc::new(RoomRegistry::<games::Pd>::new(storage.clone(), owner_id.clone()));

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

    let app = router::create_router(&config, app_state, rps_ctx, ttt_ctx, pd_ctx);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting judge server on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
