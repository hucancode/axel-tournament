// Room logic + live runner. Spec: judge/protocols/architecture.md.
//
// `RoomLogic` is the per-game contract (pure `fold` + `validate`).
// `LiveRoom<L>` is the runtime: validate -> append -> fold -> broadcast,
// with the write lock held across the sequence so concurrent ACTs see
// a consistent state.

use crate::services::storage::{Event, EventLog, LeaseStore, MetaIndex, RoomSnapshot, Storage};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Per-game logic. Pure: no I/O, no awaits.
pub trait RoomLogic: Send + Sync + 'static {
    type State: Default + Send + Sync + 'static;

    /// Apply an event already accepted by the log. Total over committed
    /// events: malformed payloads must degrade to a no-op rather than
    /// panicking, since replay must be deterministic.
    fn fold(state: &mut Self::State, kind: &str, payload: &str);

    /// Validate a proposed action. Returns the events to emit (possibly
    /// empty for a no-op) or an error string. Errors are dropped silently
    /// on the wire — see protocols/wire.md.
    fn validate(
        state: &Self::State,
        player_id: &str,
        kind: &str,
        payload: &str,
    ) -> Result<Vec<(String, String)>, String>;

    fn max_players() -> usize;
    fn game_id() -> &'static str;
    fn snapshot(state: &Self::State) -> RoomSnapshot;

    /// Players whose action is currently expected. Empty when nobody is
    /// blocking progress (lobby phase, between rounds, finished). The
    /// turn-timer watcher uses this to decide who to mark faulted on a
    /// timeout. Default: empty (turn enforcement disabled).
    fn pending_players(_state: &Self::State) -> Vec<String> {
        Vec::new()
    }
}

pub struct LiveRoom<L: RoomLogic> {
    pub room_id: String,
    pub owner_id: String,
    state: RwLock<L::State>,
    head: AtomicU64,
    last_meta: RwLock<Option<RoomSnapshot>>,
    log: Arc<dyn EventLog>,
    meta: Arc<dyn MetaIndex>,
    tx: broadcast::Sender<Event>,
}

impl<L: RoomLogic> LiveRoom<L> {
    /// Build a LiveRoom by replaying the log. Caller must hold the lease.
    pub async fn load(
        log: Arc<dyn EventLog>,
        meta: Arc<dyn MetaIndex>,
        owner_id: String,
        room_id: String,
    ) -> Result<Arc<Self>> {
        let events = log.read_since(&room_id, 0).await?;
        let mut state = L::State::default();
        let mut head = 0u64;
        for e in &events {
            L::fold(&mut state, &e.kind, &e.payload);
            head = e.seq;
        }
        let (tx, _) = broadcast::channel(1024);
        Ok(Arc::new(Self {
            room_id,
            owner_id,
            state: RwLock::new(state),
            head: AtomicU64::new(head),
            last_meta: RwLock::new(None),
            log,
            meta,
            tx,
        }))
    }

    pub fn head(&self) -> u64 {
        self.head.load(Ordering::Acquire)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    pub async fn read_since(&self, since: u64) -> Result<Vec<Event>> {
        self.log.read_since(&self.room_id, since).await
    }

    pub async fn handle_act(&self, player_id: &str, kind: &str, payload: &str) -> Result<()> {
        let mut state = self.state.write().await;
        let events_to_emit = L::validate(&state, player_id, kind, payload)
            .map_err(|e| anyhow!("invalid action: {e}"))?;

        if events_to_emit.is_empty() {
            return Ok(());
        }

        let mut last_seq = 0u64;
        for (k, p) in events_to_emit {
            let event = self.log.append(&self.room_id, &self.owner_id, &k, &p).await?;
            L::fold(&mut state, &event.kind, &event.payload);
            last_seq = event.seq;
            let _ = self.tx.send(event);
        }
        self.head.store(last_seq, Ordering::Release);
        let snap = L::snapshot(&state);
        drop(state);
        self.refresh_meta(snap, last_seq).await;
        Ok(())
    }

    /// Refresh discovery index. Skips when the snapshot is unchanged
    /// (e.g. CHAT-only events) so we don't burn an upsert per message.
    async fn refresh_meta(&self, snap: RoomSnapshot, head: u64) {
        {
            let last = self.last_meta.read().await;
            if last.as_ref() == Some(&snap) {
                return;
            }
        }
        if let Err(e) = self.meta.upsert(&self.room_id, L::game_id(), &snap, head).await {
            tracing::warn!("meta upsert failed for {}: {}", self.room_id, e);
            return;
        }
        *self.last_meta.write().await = Some(snap);
    }

    pub async fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&L::State) -> R,
    {
        let state = self.state.read().await;
        f(&state)
    }

    /// Snapshot of the current pending players. Cheap; reads under the
    /// state lock and returns the trait's projection.
    pub async fn pending_players(&self) -> Vec<String> {
        let state = self.state.read().await;
        L::pending_players(&state)
    }
}

/// Per-game room registry. Holds loaded LiveRooms keyed by room_id.
pub struct RoomRegistry<L: RoomLogic> {
    lease: Arc<dyn LeaseStore>,
    log: Arc<dyn EventLog>,
    meta: Arc<dyn MetaIndex>,
    owner_id: String,
    rooms: RwLock<HashMap<String, Arc<LiveRoom<L>>>>,
    turn_watch: Option<TurnWatchConfig>,
}

/// When set on a registry, every freshly-opened room gets a turn-timer
/// watcher attached. The callback is invoked with `(room_id, pending)`
/// after the configured timeout if the same set of players remains
/// pending. Bot/AI registries leave this `None` because match runners
/// drive their own timers.
#[derive(Clone)]
pub struct TurnWatchConfig {
    pub timeout: std::time::Duration,
    pub on_timeout: crate::services::turn_timer::TimeoutCallback,
}

impl<L: RoomLogic> RoomRegistry<L> {
    pub fn new(storage: Storage, owner_id: String) -> Self {
        Self {
            lease: storage.lease,
            log: storage.log,
            meta: storage.meta,
            owner_id,
            rooms: RwLock::new(HashMap::new()),
            turn_watch: None,
        }
    }

    pub fn with_turn_watch(mut self, cfg: TurnWatchConfig) -> Self {
        self.turn_watch = Some(cfg);
        self
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    /// Acquire lease and load the room. Returns existing handle if already
    /// loaded by this judge.
    pub async fn open(
        &self,
        room_id: &str,
        lease_ttl: std::time::Duration,
    ) -> Result<Arc<LiveRoom<L>>> {
        if let Some(room) = self.rooms.read().await.get(room_id) {
            return Ok(room.clone());
        }
        let acquired = self.lease.acquire(room_id, &self.owner_id, lease_ttl).await?;
        if !acquired {
            return Err(anyhow!("lease held by another judge"));
        }
        let room = LiveRoom::<L>::load(
            self.log.clone(),
            self.meta.clone(),
            self.owner_id.clone(),
            room_id.to_string(),
        )
        .await?;
        let mut rooms = self.rooms.write().await;
        if let Some(existing) = rooms.get(room_id) {
            return Ok(existing.clone());
        }
        rooms.insert(room_id.to_string(), room.clone());
        drop(rooms);
        let snap = room.with_state(L::snapshot).await;
        room.refresh_meta(snap, room.head()).await;
        if let Some(cfg) = &self.turn_watch {
            crate::services::turn_timer::spawn_turn_watcher(
                room.clone(),
                cfg.timeout,
                cfg.on_timeout.clone(),
            );
        }
        Ok(room)
    }

    pub async fn get(&self, room_id: &str) -> Option<Arc<LiveRoom<L>>> {
        self.rooms.read().await.get(room_id).cloned()
    }

    pub async fn drop_room(&self, room_id: &str) {
        self.rooms.write().await.remove(room_id);
        let _ = self.lease.release(room_id, &self.owner_id).await;
    }

    /// Renew leases for all loaded rooms in parallel. Drops rooms whose
    /// lease is lost.
    pub async fn heartbeat(&self, lease_ttl: std::time::Duration) {
        let room_ids: Vec<String> = self.rooms.read().await.keys().cloned().collect();
        let lease = self.lease.clone();
        let owner = self.owner_id.clone();
        let renewals = room_ids.into_iter().map(|rid| {
            let lease = lease.clone();
            let owner = owner.clone();
            async move {
                let r = lease.renew(&rid, &owner, lease_ttl).await;
                (rid, r)
            }
        });
        let results = futures_util::future::join_all(renewals).await;
        let mut to_drop: Vec<String> = Vec::new();
        for (rid, res) in results {
            match res {
                Ok(true) => {}
                Ok(false) => {
                    tracing::warn!("Lost lease for room {}, dropping", rid);
                    to_drop.push(rid);
                }
                Err(e) => tracing::error!("Heartbeat failed for room {}: {}", rid, e),
            }
        }
        if !to_drop.is_empty() {
            let mut rooms = self.rooms.write().await;
            for rid in to_drop {
                rooms.remove(&rid);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Trivial counter game: ACT INC bumps a counter.
    struct Counter;
    #[derive(Default)]
    struct CounterState {
        n: u64,
    }
    impl RoomLogic for Counter {
        type State = CounterState;
        fn fold(state: &mut Self::State, kind: &str, _payload: &str) {
            if kind == "INC" {
                state.n += 1;
            }
        }
        fn validate(
            _state: &Self::State,
            _player: &str,
            kind: &str,
            _payload: &str,
        ) -> Result<Vec<(String, String)>, String> {
            match kind {
                "INC" => Ok(vec![("INC".into(), String::new())]),
                _ => Err(format!("unknown action: {kind}")),
            }
        }
        fn max_players() -> usize {
            8
        }
        fn game_id() -> &'static str {
            "counter"
        }
        fn snapshot(_state: &Self::State) -> RoomSnapshot {
            RoomSnapshot {
                phase: "lobby".into(),
                host: None,
                players: Vec::new(),
            }
        }
    }

    #[test]
    fn counter_folds() {
        let mut s = CounterState::default();
        Counter::fold(&mut s, "INC", "");
        Counter::fold(&mut s, "INC", "");
        assert_eq!(s.n, 2);
    }

    #[test]
    fn counter_rejects_unknown() {
        let s = CounterState::default();
        let r = Counter::validate(&s, "p", "FOO", "");
        assert!(r.is_err());
    }

    /// Full pipeline (validate -> append -> fold -> broadcast) against a
    /// MemoryStorage. No database required.
    #[tokio::test]
    async fn handle_act_appends_folds_and_broadcasts() {
        let storage = Storage::memory();
        let registry = Arc::new(RoomRegistry::<Counter>::new(storage.clone(), "judge".into()));
        let room = registry.open("r", Duration::from_secs(60)).await.unwrap();
        let mut sub = room.subscribe();

        room.handle_act("alice", "INC", "").await.unwrap();
        room.handle_act("alice", "INC", "").await.unwrap();

        let e1 = sub.recv().await.unwrap();
        let e2 = sub.recv().await.unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert_eq!(room.head(), 2);

        let n = room.with_state(|s| s.n).await;
        assert_eq!(n, 2);

        // Replay reproduces state.
        let events = storage.log.read_since("r", 0).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn registry_blocks_other_judge() {
        let storage = Storage::memory();
        let r1 = Arc::new(RoomRegistry::<Counter>::new(storage.clone(), "judge-A".into()));
        let r2 = Arc::new(RoomRegistry::<Counter>::new(storage.clone(), "judge-B".into()));
        r1.open("r", Duration::from_secs(60)).await.unwrap();
        let r = r2.open("r", Duration::from_secs(60)).await;
        assert!(r.is_err(), "judge-B must be locked out while A holds lease");
    }

    /// Reload from a pre-populated log: state must reconstruct exactly.
    #[tokio::test]
    async fn load_replays_state_from_log() {
        let storage = Storage::memory();
        storage
            .lease
            .acquire("r", "seeder", Duration::from_secs(60))
            .await
            .unwrap();
        storage.log.append("r", "seeder", "INC", "").await.unwrap();
        storage.log.append("r", "seeder", "INC", "").await.unwrap();
        storage.log.append("r", "seeder", "INC", "").await.unwrap();
        storage.lease.release("r", "seeder").await.unwrap();

        let registry = Arc::new(RoomRegistry::<Counter>::new(storage.clone(), "loader".into()));
        let room = registry.open("r", Duration::from_secs(60)).await.unwrap();
        assert_eq!(room.head(), 3);
        assert_eq!(room.with_state(|s| s.n).await, 3);
    }
}
