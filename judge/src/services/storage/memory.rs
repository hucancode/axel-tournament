// In-memory storage. Tests, local dev. Same fence + monotonicity as the
// SurrealDB-backed implementation.

use super::{Event, EventLog, LeaseStore, MetaIndex, RoomMeta, RoomSnapshot};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
struct Inner {
    events: HashMap<String, Vec<Event>>,
    leases: HashMap<String, Lease>,
    meta: HashMap<String, MetaRow>,
}

struct Lease {
    owner: String,
    expires: Instant,
}

#[derive(Clone)]
struct MetaRow {
    game_id: String,
    snapshot: RoomSnapshot,
    head: u64,
    updated_at: Instant,
}

#[derive(Default)]
pub struct MemoryStorage {
    inner: Mutex<Inner>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl LeaseStore for MemoryStorage {
    async fn acquire(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool> {
        let mut g = self.inner.lock().unwrap();
        let now = Instant::now();
        let take = match g.leases.get(room) {
            None => true,
            Some(l) => l.owner == owner || l.expires < now,
        };
        if take {
            g.leases.insert(
                room.to_string(),
                Lease {
                    owner: owner.to_string(),
                    expires: now + ttl,
                },
            );
        }
        Ok(take)
    }

    async fn renew(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool> {
        let mut g = self.inner.lock().unwrap();
        let now = Instant::now();
        match g.leases.get_mut(room) {
            Some(l) if l.owner == owner => {
                l.expires = now + ttl;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn release(&self, room: &str, owner: &str) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        if matches!(g.leases.get(room), Some(l) if l.owner == owner) {
            g.leases.remove(room);
        }
        Ok(())
    }

    async fn rooms_owned_by(&self, owner: &str) -> Result<Vec<String>> {
        let g = self.inner.lock().unwrap();
        Ok(g.leases
            .iter()
            .filter(|(_, l)| l.owner == owner)
            .map(|(rid, _)| rid.clone())
            .collect())
    }
}

#[async_trait]
impl EventLog for MemoryStorage {
    async fn append(&self, room: &str, owner: &str, kind: &str, payload: &str) -> Result<Event> {
        // The Mutex never spans an .await — every method here is sync after
        // the lock — so a std Mutex inside an async trait is safe.
        let mut g = self.inner.lock().unwrap();
        match g.leases.get(room) {
            Some(l) if l.owner == owner && l.expires >= Instant::now() => {}
            _ => return Err(anyhow!("lease_lost")),
        }
        let log = g.events.entry(room.to_string()).or_default();
        let next = log.last().map(|e| e.seq).unwrap_or(0) + 1;
        let event = Event {
            seq: next,
            kind: kind.to_string(),
            payload: payload.to_string(),
        };
        log.push(event.clone());
        Ok(event)
    }

    async fn read_since(&self, room: &str, since: u64) -> Result<Vec<Event>> {
        let g = self.inner.lock().unwrap();
        let Some(log) = g.events.get(room) else { return Ok(Vec::new()) };
        // seq is monotonic per room, so `since` cuts a sorted prefix.
        let cut = log.partition_point(|e| e.seq <= since);
        Ok(log[cut..].to_vec())
    }

    async fn head(&self, room: &str) -> Result<u64> {
        let g = self.inner.lock().unwrap();
        Ok(g.events
            .get(room)
            .and_then(|log| log.last())
            .map(|e| e.seq)
            .unwrap_or(0))
    }
}

#[async_trait]
impl MetaIndex for MemoryStorage {
    async fn upsert(
        &self,
        room: &str,
        game_id: &str,
        snapshot: &RoomSnapshot,
        head: u64,
    ) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        g.meta.insert(
            room.to_string(),
            MetaRow {
                game_id: game_id.to_string(),
                snapshot: snapshot.clone(),
                head,
                updated_at: Instant::now(),
            },
        );
        Ok(())
    }

    async fn list(
        &self,
        game_id: Option<&str>,
        phase: Option<&str>,
        limit: u32,
    ) -> Result<Vec<RoomMeta>> {
        let g = self.inner.lock().unwrap();
        let mut refs: Vec<(&String, &MetaRow)> = g
            .meta
            .iter()
            .filter(|(_, m)| game_id.is_none_or(|g| g == m.game_id))
            .filter(|(_, m)| phase.is_none_or(|p| p == m.snapshot.phase))
            .collect();
        refs.sort_by(|(_, a), (_, b)| b.updated_at.cmp(&a.updated_at));
        Ok(refs
            .into_iter()
            .take(limit as usize)
            .map(|(rid, m)| RoomMeta {
                id: rid.clone(),
                game_id: m.game_id.clone(),
                phase: m.snapshot.phase.clone(),
                host: m.snapshot.host.clone(),
                players: m.snapshot.players.clone(),
                head: m.head,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> RoomSnapshot {
        RoomSnapshot {
            phase: "lobby".into(),
            host: None,
            players: Vec::new(),
        }
    }

    #[tokio::test]
    async fn lease_blocks_other_owner() {
        let s = MemoryStorage::new();
        assert!(<MemoryStorage as LeaseStore>::acquire(&s, "r", "A", Duration::from_secs(60))
            .await
            .unwrap());
        assert!(!<MemoryStorage as LeaseStore>::acquire(&s, "r", "B", Duration::from_secs(60))
            .await
            .unwrap());
        <MemoryStorage as LeaseStore>::release(&s, "r", "A").await.unwrap();
        assert!(<MemoryStorage as LeaseStore>::acquire(&s, "r", "B", Duration::from_secs(60))
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn append_assigns_monotonic_seq() {
        let s = MemoryStorage::new();
        <MemoryStorage as LeaseStore>::acquire(&s, "r", "A", Duration::from_secs(60))
            .await
            .unwrap();
        let e1 = <MemoryStorage as EventLog>::append(&s, "r", "A", "JOIN", "alice")
            .await
            .unwrap();
        let e2 = <MemoryStorage as EventLog>::append(&s, "r", "A", "JOIN", "bob")
            .await
            .unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert_eq!(<MemoryStorage as EventLog>::head(&s, "r").await.unwrap(), 2);
        let tail = <MemoryStorage as EventLog>::read_since(&s, "r", 1).await.unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].kind, "JOIN");
        assert_eq!(tail[0].payload, "bob");
    }

    #[tokio::test]
    async fn append_fenced_on_lease() {
        let s = MemoryStorage::new();
        let r = <MemoryStorage as EventLog>::append(&s, "r", "rogue", "JOIN", "alice").await;
        assert!(r.is_err(), "append without lease must fail");
    }

    #[tokio::test]
    async fn lease_expires() {
        let s = MemoryStorage::new();
        <MemoryStorage as LeaseStore>::acquire(&s, "r", "A", Duration::from_millis(10))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(<MemoryStorage as LeaseStore>::acquire(&s, "r", "B", Duration::from_secs(1))
            .await
            .unwrap());
        assert!(<MemoryStorage as EventLog>::append(&s, "r", "A", "JOIN", "x")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn meta_filters_by_game_and_phase() {
        let s = MemoryStorage::new();
        let mut snap = snap();
        <MemoryStorage as MetaIndex>::upsert(&s, "r1", "rps", &snap, 1)
            .await
            .unwrap();
        snap.phase = "playing".into();
        <MemoryStorage as MetaIndex>::upsert(&s, "r2", "rps", &snap, 5)
            .await
            .unwrap();
        <MemoryStorage as MetaIndex>::upsert(&s, "r3", "ttt", &snap, 7)
            .await
            .unwrap();

        let all = <MemoryStorage as MetaIndex>::list(&s, None, None, 10).await.unwrap();
        assert_eq!(all.len(), 3);
        let rps = <MemoryStorage as MetaIndex>::list(&s, Some("rps"), None, 10).await.unwrap();
        assert_eq!(rps.len(), 2);
        let playing = <MemoryStorage as MetaIndex>::list(&s, None, Some("playing"), 10).await.unwrap();
        assert_eq!(playing.len(), 2);
        let rps_lobby = <MemoryStorage as MetaIndex>::list(&s, Some("rps"), Some("lobby"), 10)
            .await
            .unwrap();
        assert_eq!(rps_lobby.len(), 1);
        assert_eq!(rps_lobby[0].id, "r1");
    }
}
