use std::sync::Arc;
use tokio::sync::RwLock;

/// Tracks server capacity for rooms and matches
#[derive(Debug, Clone)]
pub struct CapacityTracker {
    state: Arc<RwLock<CapacityState>>,
    max_capacity: usize,
    max_claim_delay_ms: u64,
}

#[derive(Debug)]
struct CapacityState {
    active_rooms: usize,
    active_matches: usize,
}

impl CapacityTracker {
    pub fn new(max_capacity: usize, max_claim_delay_ms: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CapacityState {
                active_rooms: 0,
                active_matches: 0,
            })),
            max_capacity,
            max_claim_delay_ms,
        }
    }

    /// Get current load (0.0 = idle, 1.0 = full capacity)
    pub async fn get_load(&self) -> f64 {
        let state = self.state.read().await;
        let total = state.active_rooms + state.active_matches;
        (total as f64) / (self.max_capacity as f64)
    }

    /// Calculate claim delay based on current load
    /// - 0% load = 0ms delay (immediate claim)
    /// - 90% load = 1000ms delay (or configured max)
    /// - 100% load = max delay
    pub async fn calculate_claim_delay(&self) -> u64 {
        let load = self.get_load().await;
        if load >= 1.0 {
            self.max_claim_delay_ms
        } else {
            (load * self.max_claim_delay_ms as f64) as u64
        }
    }

    /// Increment active match count
    pub async fn increment_matches(&self) {
        let mut state = self.state.write().await;
        state.active_matches += 1;
        tracing::debug!(
            "Match claimed. Active: {} rooms, {} matches (total: {}/{})",
            state.active_rooms,
            state.active_matches,
            state.active_rooms + state.active_matches,
            self.max_capacity
        );
    }

    /// Decrement active match count
    pub async fn decrement_matches(&self) {
        let mut state = self.state.write().await;
        state.active_matches = state.active_matches.saturating_sub(1);
        tracing::debug!(
            "Match released. Active: {} rooms, {} matches (total: {}/{})",
            state.active_rooms,
            state.active_matches,
            state.active_rooms + state.active_matches,
            self.max_capacity
        );
    }

    /// Check if server can accept more work
    pub async fn can_accept_work(&self) -> bool {
        let state = self.state.read().await;
        let total = state.active_rooms + state.active_matches;
        total < self.max_capacity
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> CapacityStats {
        let state = self.state.read().await;
        CapacityStats {
            active_rooms: state.active_rooms,
            active_matches: state.active_matches,
            total_active: state.active_rooms + state.active_matches,
            max_capacity: self.max_capacity,
            load_percentage: ((state.active_rooms + state.active_matches) as f64
                / self.max_capacity as f64
                * 100.0) as u32,
        }
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


