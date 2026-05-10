use judge::{
    app_state::AppState,
    config::Config,
    db, games, router,
    services::bot_match::{self, BotMatchHost, BotMatchRegistries},
    services::capacity::CapacityTracker,
    services::match_writer,
    services::playground,
    services::room::logic::{OnRoomOpened, RoomRegistry},
    services::room::ws::WsContext,
    services::sandbox::BuildSandbox,
    services::storage::Storage,
    services::submission,
    services::turn_timer::{self, TimeoutCallback},
};
use std::path::PathBuf;
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

    // Per-game turn-timeout from GameMetadata. Each freshly-opened
    // human room gets a watcher attached via `on_open`. Bot/AI rooms
    // skip this — match runners enforce their own turn timeout.
    let timeout_cb = match_writer::db_timeout_callback(db.clone());
    let rps_meta = games::find_game_by_id("rock-paper-scissors").expect("rps metadata");
    let ttt_meta = games::find_game_by_id("tic-tac-toe").expect("ttt metadata");
    let pd_meta = games::find_game_by_id("prisoners-dilemma").expect("pd metadata");

    let rps_registry = Arc::new(
        RoomRegistry::<games::Rps>::new(storage.clone(), owner_id.clone()).with_on_open(
            turn_watch_hook(Duration::from_millis(rps_meta.human_turn_timeout_ms), timeout_cb.clone()),
        ),
    );
    let ttt_registry = Arc::new(
        RoomRegistry::<games::Ttt>::new(storage.clone(), owner_id.clone()).with_on_open(
            turn_watch_hook(Duration::from_millis(ttt_meta.human_turn_timeout_ms), timeout_cb.clone()),
        ),
    );
    let pd_registry = Arc::new(
        RoomRegistry::<games::Pd>::new(storage.clone(), owner_id.clone()).with_on_open(
            turn_watch_hook(Duration::from_millis(pd_meta.human_turn_timeout_ms), timeout_cb),
        ),
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

    let mut bot_regs: BotMatchRegistries = BotMatchRegistries::new();
    bot_regs.insert(
        "rock-paper-scissors",
        Arc::new(rps_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_regs.insert(
        "tic-tac-toe",
        Arc::new(ttt_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_regs.insert(
        "prisoners-dilemma",
        Arc::new(pd_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_match::spawn(db.clone(), capacity.clone(), bot_regs);

    let workspace_root = std::env::var("COMPILER_WORKSPACE")
        .unwrap_or_else(|_| "/artifacts".to_string());
    let build_sandbox = Arc::new(
        BuildSandbox::new(PathBuf::from(workspace_root))
            .map_err(|e| anyhow::anyhow!("init build sandbox: {e}"))?,
    );
    submission::spawn(db.clone(), build_sandbox);

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

    let mut playground_regs: playground::PlaygroundRegistries = Default::default();
    playground_regs.insert(
        "rock-paper-scissors",
        playground::host(rps_registry.clone(), games::RpsStrategy::default),
    );
    playground_regs.insert(
        "tic-tac-toe",
        playground::host(ttt_registry.clone(), games::TttStrategy::default),
    );
    playground_regs.insert(
        "prisoners-dilemma",
        playground::host(pd_registry.clone(), games::PdStrategy::default),
    );
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

/// Spawn a turn-timer watcher each time a room is opened. Wraps the
/// glue that used to live as `TurnWatchConfig` inside `room::logic`.
fn turn_watch_hook<L: judge::services::room::logic::RoomLogic>(
    timeout: Duration,
    on_timeout: TimeoutCallback,
) -> OnRoomOpened<L> {
    Arc::new(move |room| {
        turn_timer::spawn_turn_watcher(room, timeout, on_timeout.clone());
    })
}
