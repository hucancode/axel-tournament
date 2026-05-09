// Storage traits for the room protocol.
//
// Spec: judge/protocols/architecture.md.
//
// Three independent concerns live behind three traits:
//
//   * `LeaseStore` — broker room ownership across judges.
//   * `EventLog`   — append-only per-room log; `append` is fenced on
//                    lease ownership.
//   * `MetaIndex`  — discovery view over loaded rooms; rebuildable.
//
// Each consumer takes the narrowest trait it needs, so each can be
// mocked in isolation. A `Storage` bundle is a thin convenience that
// wires three `Arc`s of the same backing store together.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::SurrealValue;

mod memory;
mod surreal;

pub use memory::MemoryStorage;
pub use surreal::SurrealStorage;

/// One committed event in a room's append-only log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SurrealValue)]
pub struct Event {
    pub seq: u64,
    pub kind: String,
    pub payload: String,
}

/// Discovery row returned by `MetaIndex::list`.
#[derive(Debug, Clone, Serialize)]
pub struct RoomMeta {
    pub id: String,
    pub game_id: String,
    pub phase: String,
    pub host: Option<String>,
    pub players: Vec<String>,
    pub head: u64,
}

/// Projection of a room's state used to refresh the discovery index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomSnapshot {
    pub phase: String,
    pub host: Option<String>,
    pub players: Vec<String>,
}

#[async_trait]
pub trait LeaseStore: Send + Sync + 'static {
    async fn acquire(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool>;
    async fn renew(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool>;
    async fn release(&self, room: &str, owner: &str) -> Result<()>;
    async fn rooms_owned_by(&self, owner: &str) -> Result<Vec<String>>;
}

#[async_trait]
pub trait EventLog: Send + Sync + 'static {
    /// Append `(kind, payload)` under `owner`'s lease.
    ///
    /// Fails atomically if the caller is not the current lease owner —
    /// this is the protocol's split-brain fence.
    async fn append(&self, room: &str, owner: &str, kind: &str, payload: &str) -> Result<Event>;
    async fn read_since(&self, room: &str, since: u64) -> Result<Vec<Event>>;
    async fn head(&self, room: &str) -> Result<u64>;
}

#[async_trait]
pub trait MetaIndex: Send + Sync + 'static {
    async fn upsert(
        &self,
        room: &str,
        game_id: &str,
        snapshot: &RoomSnapshot,
        head: u64,
    ) -> Result<()>;
    async fn list(
        &self,
        game_id: Option<&str>,
        phase: Option<&str>,
        limit: u32,
    ) -> Result<Vec<RoomMeta>>;
}

/// Bundled handle to a single backing store implementing all three
/// traits. Subsystems clone the field they need (`Arc<dyn _>`) so they
/// never see contracts they don't use.
#[derive(Clone)]
pub struct Storage {
    pub lease: Arc<dyn LeaseStore>,
    pub log: Arc<dyn EventLog>,
    pub meta: Arc<dyn MetaIndex>,
}

impl Storage {
    /// Production: SurrealDB-backed.
    pub fn surreal(db: crate::db::Database) -> Self {
        let inner = Arc::new(SurrealStorage::new(db));
        Self {
            lease: inner.clone(),
            log: inner.clone(),
            meta: inner,
        }
    }

    /// Tests / local dev: in-memory, no IO.
    pub fn memory() -> Self {
        let inner = Arc::new(MemoryStorage::new());
        Self {
            lease: inner.clone(),
            log: inner.clone(),
            meta: inner,
        }
    }
}
