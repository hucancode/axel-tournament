use crate::services::storage::{Event, EventLog, MetaIndex, RoomSnapshot};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::contract::RoomLogic;

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

    /// Server-authoritative event append. Bypasses `validate` — used by
    /// watchers (turn timer, time-pool) that need to inject terminal
    /// events the players never sent (e.g. WINNER on flag fall).
    /// Holds the state lock across append/fold/broadcast so concurrent
    /// `handle_act` calls observe the new state consistently.
    pub async fn inject_event(&self, kind: &str, payload: &str) -> Result<()> {
        let mut state = self.state.write().await;
        let event = self.log.append(&self.room_id, &self.owner_id, kind, payload).await?;
        L::fold(&mut state, &event.kind, &event.payload);
        let last_seq = event.seq;
        let _ = self.tx.send(event);
        self.head.store(last_seq, Ordering::Release);
        let snap = L::snapshot(&state);
        drop(state);
        self.refresh_meta(snap, last_seq).await;
        Ok(())
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
    /// pub(super) so `registry::open` can prime meta right after load.
    pub(super) async fn refresh_meta(&self, snap: RoomSnapshot, head: u64) {
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
