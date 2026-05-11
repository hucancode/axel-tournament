// Turn timer.
//
// Pure watcher: subscribes to a `LiveRoom`, fires a callback when the
// same set of pending players stays pending for the full timeout
// window. The callback is opaque — the room layer doesn't know how
// losses are recorded; the api or caller decides. The DB-finalising
// callback lives in `services::match_writer`.

use crate::services::room::logic::{LiveRoom, RoomLogic};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

/// Fired when the same pending player set stays pending for the full
/// timeout window. `pending` is the player ID set at fire time.
pub type TimeoutCallback =
    Arc<dyn Fn(String, Vec<String>) -> futures_util::future::BoxFuture<'static, ()> + Send + Sync>;

/// Spawn a watcher task. Returns the JoinHandle so callers can await
/// the watcher in tests; production drops it. `default_timeout` is the
/// game-wide default; `RoomLogic::per_turn_seconds` may override it
/// per-room (chess/xiangqi expose this when the host configures a
/// strict per-move clock).
pub fn spawn_turn_watcher<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    default_timeout: Duration,
    on_timeout: TimeoutCallback,
) -> tokio::task::JoinHandle<()> {
    let mut subscriber = room.subscribe();
    let room_id = room.room_id.clone();
    tokio::spawn(async move {
        let mut timeout = current_timeout::<L>(&room, default_timeout).await;
        let mut last_set: HashSet<String> = room.pending_players().await.into_iter().collect();
        let mut deadline = if last_set.is_empty() {
            None
        } else {
            Some(Instant::now() + timeout)
        };
        loop {
            tokio::select! {
                ev = subscriber.recv() => {
                    match ev {
                        Ok(_) => {
                            timeout = current_timeout::<L>(&room, default_timeout).await;
                            let now: HashSet<String> =
                                room.pending_players().await.into_iter().collect();
                            if now != last_set {
                                last_set = now;
                                deadline = if last_set.is_empty() {
                                    None
                                } else {
                                    Some(Instant::now() + timeout)
                                };
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            timeout = current_timeout::<L>(&room, default_timeout).await;
                            let now: HashSet<String> =
                                room.pending_players().await.into_iter().collect();
                            last_set = now;
                            deadline = if last_set.is_empty() {
                                None
                            } else {
                                Some(Instant::now() + timeout)
                            };
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            return;
                        }
                    }
                }
                _ = wait_until(deadline) => {
                    if !last_set.is_empty() {
                        let pending = last_set.iter().cloned().collect::<Vec<_>>();
                        on_timeout(room_id.clone(), pending).await;
                    }
                    return;
                }
            }
        }
    })
}

async fn current_timeout<L: RoomLogic>(
    room: &Arc<LiveRoom<L>>,
    default_timeout: Duration,
) -> Duration {
    match room.with_state(|s| L::per_turn_seconds(s)).await {
        Some(secs) if secs > 0 => Duration::from_secs(secs),
        _ => default_timeout,
    }
}

async fn wait_until(deadline: Option<Instant>) {
    match deadline {
        Some(t) => tokio::time::sleep_until(t).await,
        None => std::future::pending::<()>().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::room::logic::RoomRegistry;
    use crate::services::storage::{RoomSnapshot, Storage};
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::Mutex;

    /// Trivial 2-player game where ACT "MOVE" clears pending for the
    /// caller; pending starts as [a, b] after START.
    struct Pair;
    #[derive(Default)]
    struct PairState {
        started: bool,
        moved: HashSet<String>,
        players: Vec<String>,
    }
    impl RoomLogic for Pair {
        type State = PairState;
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
                }
                "MOVE" => {
                    state.moved.insert(payload.trim().to_string());
                }
                _ => {}
            }
        }
        fn validate(
            state: &Self::State,
            player: &str,
            kind: &str,
            _payload: &str,
        ) -> Result<Vec<(String, String)>, String> {
            match kind {
                "JOIN" => Ok(vec![("PLAYER_JOINED".into(), player.to_string())]),
                "START" => {
                    if !state.started && state.players.len() == 2 {
                        Ok(vec![("GAME_STARTED".into(), String::new())])
                    } else {
                        Err("nope".into())
                    }
                }
                "MOVE" => {
                    if state.started && !state.moved.contains(player) {
                        Ok(vec![("MOVE".into(), player.to_string())])
                    } else {
                        Err("nope".into())
                    }
                }
                _ => Err("unknown".into()),
            }
        }
        fn max_players() -> usize {
            2
        }
        fn game_id() -> &'static str {
            "pair"
        }
        fn snapshot(_state: &Self::State) -> RoomSnapshot {
            RoomSnapshot {
                phase: "lobby".into(),
                host: None,
                players: Vec::new(),
            }
        }
        fn pending_players(state: &Self::State) -> Vec<String> {
            if !state.started {
                return Vec::new();
            }
            state
                .players
                .iter()
                .filter(|p| !state.moved.contains(*p))
                .cloned()
                .collect()
        }
    }

    #[tokio::test(start_paused = true)]
    async fn timer_fires_with_pending_player_after_window() {
        let storage = Storage::memory();
        let registry = Arc::new(RoomRegistry::<Pair>::new(storage.clone(), "j".into()));
        let live = registry.open("r1", Duration::from_secs(60)).await.unwrap();
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "").await.unwrap();

        let captured: Arc<Mutex<Option<(String, Vec<String>)>>> = Arc::new(Mutex::new(None));
        let cap_clone = captured.clone();
        let cb: TimeoutCallback = Arc::new(move |rid, pending| {
            let cap_clone = cap_clone.clone();
            Box::pin(async move {
                *cap_clone.lock().await = Some((rid, pending));
            })
        });
        let h = spawn_turn_watcher(live, Duration::from_secs(5), cb);

        tokio::time::advance(Duration::from_secs(6)).await;
        let _ = h.await;

        let got = captured.lock().await.clone();
        let (rid, mut pending) = got.expect("timer must fire");
        pending.sort();
        assert_eq!(rid, "r1");
        assert_eq!(pending, vec!["alice".to_string(), "bob".to_string()]);
    }

    #[tokio::test(start_paused = true)]
    async fn timer_resets_when_a_player_acts() {
        let storage = Storage::memory();
        let registry = Arc::new(RoomRegistry::<Pair>::new(storage.clone(), "j2".into()));
        let live = registry.open("r2", Duration::from_secs(60)).await.unwrap();
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "").await.unwrap();

        let fires = Arc::new(AtomicU32::new(0));
        let fires_c = fires.clone();
        let cb: TimeoutCallback = Arc::new(move |_rid, _pending| {
            let fires_c = fires_c.clone();
            Box::pin(async move {
                fires_c.fetch_add(1, Ordering::SeqCst);
            })
        });
        let _h = spawn_turn_watcher(live.clone(), Duration::from_secs(5), cb);

        tokio::time::advance(Duration::from_secs(3)).await;
        live.handle_act("alice", "MOVE", "").await.unwrap();
        tokio::time::advance(Duration::from_secs(3)).await;
        tokio::task::yield_now().await;
        assert_eq!(fires.load(Ordering::SeqCst), 0);

        tokio::time::advance(Duration::from_secs(5)).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
        assert_eq!(fires.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn timer_idle_when_pending_set_is_empty() {
        let storage = Storage::memory();
        let registry = Arc::new(RoomRegistry::<Pair>::new(storage.clone(), "j3".into()));
        let live = registry.open("r3", Duration::from_secs(60)).await.unwrap();
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();

        let fires = Arc::new(AtomicU32::new(0));
        let fires_c = fires.clone();
        let cb: TimeoutCallback = Arc::new(move |_rid, _pending| {
            let fires_c = fires_c.clone();
            Box::pin(async move {
                fires_c.fetch_add(1, Ordering::SeqCst);
            })
        });
        let _h = spawn_turn_watcher(live, Duration::from_secs(5), cb);
        tokio::time::advance(Duration::from_secs(60)).await;
        tokio::task::yield_now().await;
        assert_eq!(
            fires.load(Ordering::SeqCst),
            0,
            "no pending players => timer must not fire"
        );
    }
}
