// Pool-based time control watcher.
//
// Subscribes to a `LiveRoom`, accumulates per-player active time while
// the player is in `pending_players()`, and injects a `WINNER` event for
// the opponent the moment a player's pool budget is exhausted.
//
// The match-finalizer watcher (services::match_finalizer) folds that
// WINNER like any other terminal event, so DB writeback + ELO + room
// status updates run through the same path as a normal mate.
//
// Pool size comes from `RoomLogic::time_pool_seconds`; games that don't
// support pool clocks (default impl returns `None`) skip this watcher.

use crate::services::room::logic::{LiveRoom, RoomLogic};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

/// Spawn a watcher task. Returns the JoinHandle so callers can await
/// the watcher in tests; production drops it. The task exits silently
/// if the room has no pool budget yet (still in lobby) — we re-check
/// every event so a `GAME_STARTED` carrying a fresh config kicks it on.
pub fn spawn_time_pool_watcher<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        run::<L>(room).await;
    })
}

async fn run<L: RoomLogic>(room: Arc<LiveRoom<L>>) {
    let mut subscriber = room.subscribe();
    // Per-player accumulated active time in milliseconds.
    let mut used_ms: HashMap<String, u64> = HashMap::new();
    // Players currently being clocked, with the wall-clock instant they
    // last became pending.
    let mut active_since: HashMap<String, Instant> = HashMap::new();
    // Cached: roster + pool budget. Refreshed on each event so a
    // post-START config update or game restart takes effect.
    let (mut roster, mut budget_ms) = read_config::<L>(&room).await;

    let mut pending: Vec<String> = room.pending_players().await;
    let now = Instant::now();
    for p in &pending {
        active_since.insert(p.clone(), now);
    }

    loop {
        let deadline = next_deadline(&active_since, &used_ms, budget_ms);

        tokio::select! {
            ev = subscriber.recv() => {
                if ev.is_err() {
                    return;
                }
                let (r, b) = read_config::<L>(&room).await;
                roster = r;
                budget_ms = b;
                let new_pending: Vec<String> = room.pending_players().await;
                let now = Instant::now();
                stop_clocks(&mut used_ms, &mut active_since, &pending, &new_pending, now);
                start_clocks(&mut active_since, &pending, &new_pending, now);
                pending = new_pending;
            }
            _ = sleep_until(deadline) => {
                let now = Instant::now();
                let Some(budget) = budget_ms else { continue };
                if let Some(loser) = first_flagged(&active_since, &used_ms, budget, now) {
                    let loser_idx = roster.iter().position(|p| p == &loser);
                    let opponent_idx = match (loser_idx, roster.len()) {
                        (Some(i), n) if n >= 2 => Some((i + 1) % n),
                        _ => None,
                    };
                    if let Some(idx) = opponent_idx {
                        if let Err(e) = room
                            .inject_event("WINNER", &idx.to_string())
                            .await
                        {
                            tracing::warn!("time_pool: inject WINNER failed: {e:#}");
                        }
                    } else {
                        tracing::warn!(
                            "time_pool: flag without opponent (room={}, loser={loser})",
                            room.room_id
                        );
                    }
                    return;
                }
            }
        }
    }
}

async fn read_config<L: RoomLogic>(
    room: &Arc<LiveRoom<L>>,
) -> (Vec<String>, Option<u64>) {
    room.with_state(|s| {
        let roster = L::snapshot(s).players;
        let budget = L::time_pool_seconds(s).map(|sec| sec.saturating_mul(1000));
        (roster, budget)
    })
    .await
}

fn stop_clocks(
    used_ms: &mut HashMap<String, u64>,
    active_since: &mut HashMap<String, Instant>,
    old_pending: &[String],
    new_pending: &[String],
    now: Instant,
) {
    for p in old_pending {
        if !new_pending.contains(p) {
            if let Some(start) = active_since.remove(p) {
                let elapsed = now.saturating_duration_since(start).as_millis() as u64;
                *used_ms.entry(p.clone()).or_insert(0) += elapsed;
            }
        }
    }
}

fn start_clocks(
    active_since: &mut HashMap<String, Instant>,
    old_pending: &[String],
    new_pending: &[String],
    now: Instant,
) {
    for p in new_pending {
        if !old_pending.contains(p) {
            active_since.insert(p.clone(), now);
        }
    }
}

fn next_deadline(
    active_since: &HashMap<String, Instant>,
    used_ms: &HashMap<String, u64>,
    budget_ms: Option<u64>,
) -> Option<Instant> {
    let budget = budget_ms?;
    active_since
        .iter()
        .map(|(pid, since)| {
            let used = *used_ms.get(pid).unwrap_or(&0);
            let remaining = budget.saturating_sub(used);
            *since + Duration::from_millis(remaining)
        })
        .min()
}

fn first_flagged(
    active_since: &HashMap<String, Instant>,
    used_ms: &HashMap<String, u64>,
    budget_ms: u64,
    now: Instant,
) -> Option<String> {
    active_since
        .iter()
        .find_map(|(pid, since)| {
            let used = *used_ms.get(pid).unwrap_or(&0);
            let elapsed = now.saturating_duration_since(*since).as_millis() as u64;
            if used + elapsed >= budget_ms {
                Some(pid.clone())
            } else {
                None
            }
        })
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(t) => tokio::time::sleep_until(t).await,
        None => std::future::pending::<()>().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::room::logic::{RoomLogic, RoomRegistry};
    use crate::services::storage::{RoomSnapshot, Storage};
    use std::collections::HashSet;

    /// Trivial 2-player game: ACT START flips `started`, MOVE clears
    /// caller from `pending`. Pool budget configurable per-room via the
    /// initial GAME_STARTED payload (`<seconds>`). Default = no pool.
    struct PoolPair;
    #[derive(Default)]
    struct PoolState {
        started: bool,
        pool_seconds: u64,
        moved: HashSet<String>,
        players: Vec<String>,
        winner: Option<u8>,
    }
    impl RoomLogic for PoolPair {
        type State = PoolState;
        fn fold(state: &mut Self::State, kind: &str, payload: &str) {
            match kind {
                "PLAYER_JOINED" => {
                    let pid = payload.trim().to_string();
                    if !state.players.contains(&pid) {
                        state.players.push(pid);
                    }
                }
                "GAME_STARTED" => {
                    state.started = true;
                    state.moved.clear();
                    state.pool_seconds =
                        payload.trim().parse::<u64>().unwrap_or(0);
                }
                "MOVE" => {
                    state.moved.insert(payload.trim().to_string());
                    if state.moved.len() == state.players.len() {
                        // round complete; next round
                        state.moved.clear();
                    }
                }
                "WINNER" => {
                    if let Ok(w) = payload.trim().parse::<u8>() {
                        state.winner = Some(w);
                    }
                }
                _ => {}
            }
        }
        fn validate(
            state: &Self::State,
            player: &str,
            kind: &str,
            payload: &str,
        ) -> Result<Vec<(String, String)>, String> {
            match kind {
                "JOIN" => Ok(vec![("PLAYER_JOINED".into(), player.to_string())]),
                "START" => {
                    if !state.started && state.players.len() == 2 {
                        Ok(vec![("GAME_STARTED".into(), payload.trim().to_string())])
                    } else {
                        Err("nope".into())
                    }
                }
                "MOVE" => Ok(vec![("MOVE".into(), player.to_string())]),
                _ => Err("unknown".into()),
            }
        }
        fn max_players() -> usize {
            2
        }
        fn game_id() -> &'static str {
            "pool-pair"
        }
        fn snapshot(state: &Self::State) -> RoomSnapshot {
            RoomSnapshot {
                phase: if state.winner.is_some() {
                    "finished"
                } else if state.started {
                    "playing"
                } else {
                    "lobby"
                }
                .into(),
                host: state.players.first().cloned(),
                players: state.players.clone(),
            }
        }
        fn pending_players(state: &Self::State) -> Vec<String> {
            if !state.started || state.winner.is_some() {
                return Vec::new();
            }
            state
                .players
                .iter()
                .filter(|p| !state.moved.contains(*p))
                .cloned()
                .collect()
        }
        fn time_pool_seconds(state: &Self::State) -> Option<u64> {
            if state.pool_seconds == 0 {
                None
            } else {
                Some(state.pool_seconds)
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn flags_player_when_pool_exhausted() {
        let storage = Storage::memory();
        let registry =
            Arc::new(RoomRegistry::<PoolPair>::new(storage.clone(), "j".into()));
        let live = registry.open("rA", Duration::from_secs(60)).await.unwrap();
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "5").await.unwrap();

        let _h = spawn_time_pool_watcher(live.clone());
        // Let the watcher poll, subscribe, and record start instants.
        tokio::task::yield_now().await;

        // Both pending; advance past 5s pool. Whichever the HashMap
        // picks first flags; the watcher emits WINNER for opponent.
        tokio::time::advance(Duration::from_secs(6)).await;
        tokio::time::sleep(Duration::from_millis(1)).await;

        let winner = live.with_state(|s| s.winner).await;
        assert!(winner.is_some(), "WINNER must have been injected");
        assert!(matches!(winner, Some(0) | Some(1)));
    }

    #[tokio::test(start_paused = true)]
    async fn no_pool_means_no_flag() {
        let storage = Storage::memory();
        let registry =
            Arc::new(RoomRegistry::<PoolPair>::new(storage.clone(), "j".into()));
        let live = registry.open("rB", Duration::from_secs(60)).await.unwrap();
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "0").await.unwrap();

        let _h = spawn_time_pool_watcher(live.clone());
        tokio::time::advance(Duration::from_secs(3600)).await;
        tokio::task::yield_now().await;

        let winner = live.with_state(|s| s.winner).await;
        assert!(winner.is_none(), "no pool budget => no flag");
    }

    #[tokio::test(start_paused = true)]
    async fn moving_freezes_clock_for_caller() {
        let storage = Storage::memory();
        let registry =
            Arc::new(RoomRegistry::<PoolPair>::new(storage.clone(), "j".into()));
        let live = registry.open("rC", Duration::from_secs(60)).await.unwrap();
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "10").await.unwrap();

        let _h = spawn_time_pool_watcher(live.clone());
        tokio::task::yield_now().await;

        // Alice moves at t=2s, then we let bob keep ticking. Bob should
        // flag at his own ~10s pool, not alice's 8s remaining.
        tokio::time::advance(Duration::from_secs(2)).await;
        live.handle_act("alice", "MOVE", "").await.unwrap();
        tokio::time::sleep(Duration::from_millis(1)).await;
        // Bob keeps ticking; cross his 10s mark.
        tokio::time::advance(Duration::from_secs(11)).await;
        tokio::time::sleep(Duration::from_millis(1)).await;

        let winner = live.with_state(|s| s.winner).await;
        // Bob (idx 1) flags, so opponent (alice, idx 0) wins.
        assert_eq!(winner, Some(0));
    }
}
