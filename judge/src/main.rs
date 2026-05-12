use judge::{
    app_state::AppState,
    config::Config,
    db, games, router,
    services::bot_match::{self, BotMatchHost, BotMatchRegistries},
    services::capacity::CapacityTracker,
    services::match_finalizer::{self, FinishCallback},
    services::match_writer,
    services::playground,
    services::playground_submission,
    services::room::logic::{OnRoomOpened, RoomRegistry},
    services::room::ws::WsContext,
    services::sandbox::BuildSandbox,
    services::storage::Storage,
    services::submission,
    services::time_pool,
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
    let finish_cb = match_writer::db_finish_callback(db.clone());

    let rps_registry = build_registry::<games::Rps>(
        &storage, &owner_id, "rock-paper-scissors", false,
        timeout_cb.clone(), finish_cb.clone(),
    );
    let ttt_registry = build_registry::<games::Ttt>(
        &storage, &owner_id, "tic-tac-toe", false,
        timeout_cb.clone(), finish_cb.clone(),
    );
    let pd_registry = build_registry::<games::Pd>(
        &storage, &owner_id, "prisoners-dilemma", false,
        timeout_cb.clone(), finish_cb.clone(),
    );
    let chess_registry = build_registry::<games::Chess>(
        &storage, &owner_id, "chess", true,
        timeout_cb.clone(), finish_cb.clone(),
    );
    let xiangqi_registry = build_registry::<games::Xiangqi>(
        &storage, &owner_id, "xiangqi", true,
        timeout_cb.clone(), finish_cb.clone(),
    );
    let poker_registry = build_registry::<games::Poker>(
        &storage, &owner_id, "poker", false,
        timeout_cb.clone(), finish_cb.clone(),
    );
    let jog_registry = build_registry::<games::Jog>(
        &storage, &owner_id, "jar-of-greed", false,
        timeout_cb, finish_cb,
    );

    let rps_ctx = build_ctx(rps_registry.clone(), &config.jwt_secret);
    let ttt_ctx = build_ctx(ttt_registry.clone(), &config.jwt_secret);
    let pd_ctx = build_ctx(pd_registry.clone(), &config.jwt_secret);
    let chess_ctx = build_ctx(chess_registry.clone(), &config.jwt_secret);
    let xiangqi_ctx = build_ctx(xiangqi_registry.clone(), &config.jwt_secret);
    let poker_ctx = build_ctx(poker_registry.clone(), &config.jwt_secret);
    let jog_ctx = build_ctx(jog_registry.clone(), &config.jwt_secret);

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
    bot_regs.insert(
        "chess",
        Arc::new(chess_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_regs.insert(
        "xiangqi",
        Arc::new(xiangqi_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_regs.insert(
        "poker",
        Arc::new(poker_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_regs.insert(
        "jar-of-greed",
        Arc::new(jog_registry.clone()) as Arc<dyn BotMatchHost>,
    );
    bot_match::spawn(db.clone(), capacity.clone(), bot_regs);

    let workspace_root = std::env::var("COMPILER_WORKSPACE")
        .unwrap_or_else(|_| "/artifacts".to_string());
    let build_sandbox = Arc::new(
        BuildSandbox::new(PathBuf::from(workspace_root))
            .map_err(|e| anyhow::anyhow!("init build sandbox: {e}"))?,
    );
    submission::spawn(db.clone(), build_sandbox);

    let mut supervisor = axel_core::tasks::Supervisor::new();
    let hb_rps = rps_registry.clone();
    let hb_ttt = ttt_registry.clone();
    let hb_pd = pd_registry.clone();
    let hb_chess = chess_registry.clone();
    let hb_xiangqi = xiangqi_registry.clone();
    let hb_poker = poker_registry.clone();
    let hb_jog = jog_registry.clone();
    supervisor.spawn_forever("room-heartbeat", move || {
        let hb_rps = hb_rps.clone();
        let hb_ttt = hb_ttt.clone();
        let hb_pd = hb_pd.clone();
        let hb_chess = hb_chess.clone();
        let hb_xiangqi = hb_xiangqi.clone();
        let hb_poker = hb_poker.clone();
        let hb_jog = hb_jog.clone();
        async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            loop {
                tick.tick().await;
                tokio::join!(
                    hb_rps.heartbeat(Duration::from_secs(15)),
                    hb_ttt.heartbeat(Duration::from_secs(15)),
                    hb_pd.heartbeat(Duration::from_secs(15)),
                    hb_chess.heartbeat(Duration::from_secs(15)),
                    hb_xiangqi.heartbeat(Duration::from_secs(15)),
                    hb_poker.heartbeat(Duration::from_secs(15)),
                    hb_jog.heartbeat(Duration::from_secs(15)),
                );
            }
        }
    });

    let mut submission_playground_regs: playground_submission::SubmissionPlaygroundRegistries =
        Default::default();
    submission_playground_regs.insert(
        "rock-paper-scissors",
        playground_submission::host(rps_registry.clone()),
    );
    submission_playground_regs.insert(
        "tic-tac-toe",
        playground_submission::host(ttt_registry.clone()),
    );
    submission_playground_regs.insert(
        "prisoners-dilemma",
        playground_submission::host(pd_registry.clone()),
    );
    submission_playground_regs.insert(
        "chess",
        playground_submission::host(chess_registry.clone()),
    );
    submission_playground_regs.insert(
        "xiangqi",
        playground_submission::host(xiangqi_registry.clone()),
    );
    submission_playground_regs.insert(
        "poker",
        playground_submission::host(poker_registry.clone()),
    );
    submission_playground_regs.insert(
        "jar-of-greed",
        playground_submission::host(jog_registry.clone()),
    );

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
    playground_regs.insert(
        "chess",
        playground::host(chess_registry.clone(), games::ChessStrategy::default),
    );
    playground_regs.insert(
        "xiangqi",
        playground::host(xiangqi_registry.clone(), games::XiangqiStrategy::default),
    );
    playground_regs.insert(
        "poker",
        playground::host(poker_registry.clone(), games::PokerStrategy::default),
    );
    playground_regs.insert(
        "jar-of-greed",
        playground::host(jog_registry.clone(), games::JogStrategy::default),
    );
    let app = router::create_router(
        &config,
        app_state,
        rps_ctx,
        ttt_ctx,
        pd_ctx,
        chess_ctx,
        xiangqi_ctx,
        poker_ctx,
        jog_ctx,
        playground_regs,
        submission_playground_regs,
    );

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting judge server on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Spawn turn-timer + match-finalizer watchers each time a room is
/// opened. Together they cover the two ways a human room ends:
///   * turn-timer fires on per-turn timeout (player went silent);
///   * match-finalizer fires on a terminal event (WINNER/DRAW/GAME_END).
/// Both write the match row + flip room status; the api-side healer
/// then runs ELO + tournament finalization.
fn room_open_hook<L: judge::services::room::logic::RoomLogic>(
    timeout: Duration,
    on_timeout: TimeoutCallback,
    on_finish: FinishCallback,
) -> OnRoomOpened<L> {
    Arc::new(move |room| {
        turn_timer::spawn_turn_watcher(room.clone(), timeout, on_timeout.clone());
        match_finalizer::spawn_finish_watcher(room, on_finish.clone());
    })
}

/// Same as `room_open_hook` plus a pool-clock watcher. Used by chess +
/// xiangqi where the host can pick a per-player time pool; games that
/// don't override `time_pool_seconds` see the watcher sit idle.
fn room_open_hook_with_pool<L: judge::services::room::logic::RoomLogic>(
    timeout: Duration,
    on_timeout: TimeoutCallback,
    on_finish: FinishCallback,
) -> OnRoomOpened<L> {
    Arc::new(move |room| {
        turn_timer::spawn_turn_watcher(room.clone(), timeout, on_timeout.clone());
        time_pool::spawn_time_pool_watcher(room.clone());
        match_finalizer::spawn_finish_watcher(room, on_finish.clone());
    })
}

/// One-shot per-game registry construction. Looks up turn-timeout from
/// GameMetadata, wires the appropriate room-open hook (with or without
/// pool clock), and returns the Arc-wrapped registry ready for use.
fn build_registry<L: judge::services::room::logic::RoomLogic>(
    storage: &judge::services::storage::Storage,
    owner_id: &str,
    game_id: &'static str,
    with_pool: bool,
    timeout_cb: TimeoutCallback,
    finish_cb: FinishCallback,
) -> Arc<RoomRegistry<L>> {
    let timeout = Duration::from_millis(
        games::find_game_by_id(game_id)
            .unwrap_or_else(|| panic!("{game_id} metadata"))
            .human_turn_timeout_ms,
    );
    let hook = if with_pool {
        room_open_hook_with_pool::<L>(timeout, timeout_cb, finish_cb)
    } else {
        room_open_hook::<L>(timeout, timeout_cb, finish_cb)
    };
    Arc::new(
        RoomRegistry::<L>::new(storage.clone(), owner_id.to_string()).with_on_open(hook),
    )
}

fn build_ctx<L: judge::services::room::logic::RoomLogic>(
    registry: Arc<RoomRegistry<L>>,
    jwt_secret: &str,
) -> Arc<WsContext<L>> {
    Arc::new(WsContext {
        registry,
        jwt_secret: jwt_secret.to_string(),
    })
}
