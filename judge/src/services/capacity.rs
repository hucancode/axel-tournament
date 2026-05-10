// Capacity tracker: counts active rooms and matches against a global
// budget. Mutators are RAII guards (`RoomGuard`/`MatchGuard`) so the
// caller can't leak a slot by forgetting to decrement.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CapacityTracker {
    state: Arc<State>,
}

#[derive(Debug)]
struct State {
    active_rooms: AtomicUsize,
    active_matches: AtomicUsize,
    max_capacity: usize,
    max_claim_delay_ms: u64,
}

impl CapacityTracker {
    pub fn new(max_capacity: usize, max_claim_delay_ms: u64) -> Self {
        Self {
            state: Arc::new(State {
                active_rooms: AtomicUsize::new(0),
                active_matches: AtomicUsize::new(0),
                max_capacity,
                max_claim_delay_ms,
            }),
        }
    }

    pub fn snapshot(&self) -> CapacityStats {
        let rooms = self.state.active_rooms.load(Ordering::Relaxed);
        let matches = self.state.active_matches.load(Ordering::Relaxed);
        let max = self.state.max_capacity;
        CapacityStats {
            active_rooms: rooms,
            active_matches: matches,
            total_active: rooms + matches,
            max_capacity: max,
            load_percentage: if max == 0 {
                0
            } else {
                ((rooms + matches) as f64 / max as f64 * 100.0) as u32
            },
        }
    }

    /// 0.0 idle .. 1.0 saturated.
    pub fn load(&self) -> f64 {
        let snap = self.snapshot();
        if snap.max_capacity == 0 {
            return 0.0;
        }
        snap.total_active as f64 / snap.max_capacity as f64
    }

    /// Backoff delay scaled by load. Saturates at `max_claim_delay_ms`.
    pub fn claim_delay_ms(&self) -> u64 {
        let load = self.load();
        if load >= 1.0 {
            self.state.max_claim_delay_ms
        } else {
            (load * self.state.max_claim_delay_ms as f64) as u64
        }
    }

    pub fn can_accept_work(&self) -> bool {
        self.snapshot().total_active < self.state.max_capacity
    }

    /// Acquire a room slot. Drop the returned guard to release.
    pub fn room_slot(&self) -> RoomGuard {
        self.state.active_rooms.fetch_add(1, Ordering::Relaxed);
        self.log_change("Room created");
        RoomGuard {
            state: self.state.clone(),
        }
    }

    /// Acquire a match slot. Drop the returned guard to release.
    pub fn match_slot(&self) -> MatchGuard {
        self.state.active_matches.fetch_add(1, Ordering::Relaxed);
        self.log_change("Match claimed");
        MatchGuard {
            state: self.state.clone(),
        }
    }

    fn log_change(&self, event: &str) {
        let snap = self.snapshot();
        tracing::debug!(
            "{event}. Active: {} rooms, {} matches (total: {}/{})",
            snap.active_rooms,
            snap.active_matches,
            snap.total_active,
            snap.max_capacity
        );
    }
}

pub struct RoomGuard {
    state: Arc<State>,
}

impl Drop for RoomGuard {
    fn drop(&mut self) {
        self.state
            .active_rooms
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_sub(1))
            })
            .ok();
    }
}

pub struct MatchGuard {
    state: Arc<State>,
}

impl Drop for MatchGuard {
    fn drop(&mut self) {
        self.state
            .active_matches
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_sub(1))
            })
            .ok();
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CapacityStats {
    pub active_rooms: usize,
    pub active_matches: usize,
    pub total_active: usize,
    pub max_capacity: usize,
    pub load_percentage: u32,
}
