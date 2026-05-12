use crate::services::storage::{EventLog, LeaseStore, MetaIndex, Storage};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::contract::RoomLogic;
use super::live::LiveRoom;

/// Per-room hook fired after a room is freshly loaded into the
/// registry. Used by the human-vs-human flow to attach the turn-timer
/// watcher; AI registries leave this unset.
pub type OnRoomOpened<L> = Arc<dyn Fn(Arc<LiveRoom<L>>) + Send + Sync>;

/// Per-game room registry. Holds loaded LiveRooms keyed by room_id.
pub struct RoomRegistry<L: RoomLogic> {
    lease: Arc<dyn LeaseStore>,
    log: Arc<dyn EventLog>,
    meta: Arc<dyn MetaIndex>,
    owner_id: String,
    rooms: RwLock<HashMap<String, Arc<LiveRoom<L>>>>,
    on_open: Option<OnRoomOpened<L>>,
}

impl<L: RoomLogic> RoomRegistry<L> {
    pub fn new(storage: Storage, owner_id: String) -> Self {
        Self {
            lease: storage.lease,
            log: storage.log,
            meta: storage.meta,
            owner_id,
            rooms: RwLock::new(HashMap::new()),
            on_open: None,
        }
    }

    pub fn with_on_open(mut self, hook: OnRoomOpened<L>) -> Self {
        self.on_open = Some(hook);
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
        if let Some(hook) = &self.on_open {
            hook(room.clone());
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
