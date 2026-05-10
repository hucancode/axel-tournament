// In-process playground bot. The protocol-learning sandbox spawns a
// real `LiveRoom` and attaches a deterministic sample bot inside the
// judge process — no subprocess, no JWT, just a synthetic player_id
// (`bot:sample`) calling `LiveRoom::handle_act` directly.
//
// Per-game dispatch is via a `HashMap<game_id, Arc<dyn PlaygroundHost>>`,
// the same pattern `bot_match` uses for bot tournaments. Strategies live
// alongside their game logic in `crate::games`.

use crate::services::room::logic::{LiveRoom, RoomLogic, RoomRegistry};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;

const PLAYGROUND_LEASE_TTL: Duration = Duration::from_secs(15);
const BOT_PLAYER_ID: &str = "bot:sample";

/// Strategy interface: react to one event and (optionally) emit one ACT.
/// Implementations live in `crate::games` next to the matching `RoomLogic`.
pub trait PlaygroundStrategy: Send + 'static {
    fn on_attach(&mut self, _bot_pid: &str) {}

    /// React to a freshly committed event. `Some((kind, payload))`
    /// emits an ACT; `None` waits for the next event.
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)>;
}

/// Erased per-game spawn entry. `spawn` opens the room, attaches the
/// game-specific strategy, and returns the bot's player id.
#[async_trait]
pub trait PlaygroundHost: Send + Sync + 'static {
    async fn spawn(&self, room_id: String) -> Result<String>;
}

pub type PlaygroundRegistries = HashMap<&'static str, Arc<dyn PlaygroundHost>>;

/// Construct a `PlaygroundHost` from a registry + strategy factory.
pub fn host<L, S, F>(registry: Arc<RoomRegistry<L>>, factory: F) -> Arc<dyn PlaygroundHost>
where
    L: RoomLogic,
    S: PlaygroundStrategy,
    F: Fn() -> S + Send + Sync + 'static,
{
    Arc::new(GameHost { registry, factory })
}

struct GameHost<L: RoomLogic, S: PlaygroundStrategy, F: Fn() -> S + Send + Sync + 'static> {
    registry: Arc<RoomRegistry<L>>,
    factory: F,
}

#[async_trait]
impl<L, S, F> PlaygroundHost for GameHost<L, S, F>
where
    L: RoomLogic,
    S: PlaygroundStrategy,
    F: Fn() -> S + Send + Sync + 'static,
{
    async fn spawn(&self, room_id: String) -> Result<String> {
        let room = self.registry.open(&room_id, PLAYGROUND_LEASE_TTL).await?;
        let strategy = (self.factory)();
        spawn_driver(self.registry.clone(), room, room_id, strategy);
        Ok(BOT_PLAYER_ID.to_string())
    }
}

fn spawn_driver<L, S>(
    registry: Arc<RoomRegistry<L>>,
    room: Arc<LiveRoom<L>>,
    room_id: String,
    mut strategy: S,
) where
    L: RoomLogic,
    S: PlaygroundStrategy,
{
    tokio::spawn(async move {
        if let Err(e) = drive(room, &mut strategy).await {
            tracing::debug!("playground bot exited: {e}");
        }
        registry.drop_room(&room_id).await;
    });
}

async fn drive<L, S>(room: Arc<LiveRoom<L>>, strategy: &mut S) -> Result<()>
where
    L: RoomLogic,
    S: PlaygroundStrategy,
{
    let bot_pid = BOT_PLAYER_ID.to_string();
    strategy.on_attach(&bot_pid);

    let mut sub = room.subscribe();

    let backlog = room.read_since(0).await?;
    let mut joined = false;
    for e in &backlog {
        if !joined && e.kind == "PLAYER_JOINED" && e.payload.trim() != bot_pid {
            let _ = room.handle_act(&bot_pid, "JOIN", "").await;
            joined = true;
        }
        if let Some((kind, payload)) = strategy.react(&bot_pid, &e.kind, &e.payload) {
            let _ = room.handle_act(&bot_pid, &kind, &payload).await;
        }
    }

    loop {
        match sub.recv().await {
            Ok(event) => {
                if !joined && event.kind == "PLAYER_JOINED" && event.payload.trim() != bot_pid {
                    let _ = room.handle_act(&bot_pid, "JOIN", "").await;
                    joined = true;
                }
                if event.kind == "GAME_END"
                    || event.kind == "WINNER"
                    || event.kind == "DRAW"
                {
                    return Ok(());
                }
                if let Some((kind, payload)) =
                    strategy.react(&bot_pid, &event.kind, &event.payload)
                {
                    let _ = room.handle_act(&bot_pid, &kind, &payload).await;
                }
            }
            Err(RecvError::Lagged(n)) => {
                tracing::warn!("playground bot lagged by {n}; continuing");
            }
            Err(RecvError::Closed) => return Err(anyhow!("event stream closed")),
        }
    }
}
