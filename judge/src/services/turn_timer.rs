// Turn timer.
//
// Watches a `LiveRoom` and fires a callback when the current pending
// players go too long without acting. Used by the human-vs-human flow
// to enforce "miss your turn = lose" semantics. The callback is opaque:
// the room layer doesn't know how losses are recorded; the api or
// caller decides.

use crate::services::room::logic::{LiveRoom, RoomLogic};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

/// Fired when the same set of pending players stays pending for the
/// full timeout window. `pending` is the player ID set at fire time;
/// callers should mark these players faulted.
pub type TimeoutCallback =
    Arc<dyn Fn(String, Vec<String>) -> futures_util::future::BoxFuture<'static, ()> + Send + Sync>;

/// Default timeout writer: marks the room finished and inserts a match
/// row attributing the loss to the pending players. Surviving player
/// (if exactly one non-pending member remains) becomes winner; if all
/// players are pending, no winner is declared.
pub fn db_timeout_callback(db: crate::db::Database) -> TimeoutCallback {
    Arc::new(move |room_id, pending| {
        let db = db.clone();
        Box::pin(async move {
            if let Err(e) = write_timeout_match(&db, &room_id, &pending).await {
                tracing::warn!("turn timeout writer failed for {room_id}: {e:#}");
            }
        })
    })
}

async fn write_timeout_match(
    db: &crate::db::Database,
    room_id: &str,
    pending: &[String],
) -> anyhow::Result<()> {
    use surrealdb::types::{SurrealValue, ToSql};

    #[derive(serde::Deserialize, SurrealValue)]
    struct RoomRow {
        players: Vec<surrealdb::types::RecordId>,
        #[serde(default)]
        tournament_id: Option<surrealdb::types::RecordId>,
        game_id: String,
        status: String,
    }

    // Fetch the room. `room_id` may be either a fully-qualified
    // `room:abc` or a bare `abc`; normalise to record id.
    let rid = parse_room_id(room_id);
    let mut resp = db
        .query("SELECT players, tournament_id, game_id, status FROM $rid")
        .bind(("rid", rid.clone()))
        .await?;
    let rows: Vec<RoomRow> = match resp.take(0) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("turn_timeout: room decode failed for {room_id}: {e}");
            return Ok(());
        }
    };
    let Some(room) = rows.into_iter().next() else {
        // Room not in api db (probably a transient AI-only room) — nothing to do.
        tracing::debug!("turn_timeout: no room found for {room_id}, skipping");
        return Ok(());
    };
    if room.status == "finished" {
        return Ok(());
    }

    let pending_set: std::collections::HashSet<String> =
        pending.iter().cloned().collect();
    let is_pending = |p: &surrealdb::types::RecordId| pending_set.contains(&p.to_sql());
    let pending_records: Vec<surrealdb::types::RecordId> =
        room.players.iter().filter(|p| is_pending(p)).cloned().collect();
    let surviving: Vec<&surrealdb::types::RecordId> =
        room.players.iter().filter(|p| !is_pending(p)).collect();
    let winner = if surviving.len() == 1 {
        Some(surviving[0].clone())
    } else {
        None
    };

    // Build participant rows: faulted players score 0, winner 1, others 0.5.
    #[derive(serde::Serialize, SurrealValue)]
    struct Part {
        user_id: surrealdb::types::RecordId,
        submission_id: Option<surrealdb::types::RecordId>,
        score: Option<f64>,
    }
    let participants: Vec<Part> = room
        .players
        .iter()
        .map(|p| {
            let score = if Some(p) == winner.as_ref() {
                Some(1.0)
            } else if is_pending(p) {
                Some(0.0)
            } else {
                Some(0.5)
            };
            Part {
                user_id: p.clone(),
                submission_id: None,
                score,
            }
        })
        .collect();

    db.query(
        "CREATE match SET
             tournament_id = $tid,
             game_id = $gid,
             room_id = $rid,
             status = 'completed',
             participants = $parts,
             error_message = 'turn_timeout',
             faulted_user_ids = $faulted,
             created_at = time::now(),
             updated_at = time::now(),
             started_at = time::now(),
             completed_at = time::now();",
    )
    .bind(("tid", room.tournament_id.clone()))
    .bind(("gid", room.game_id.clone()))
    .bind(("rid", rid.clone()))
    .bind(("parts", participants))
    .bind(("faulted", pending_records))
    .await?;

    db.query(
        "UPDATE $rid SET status = 'finished',
                          winner_id = $winner,
                          updated_at = time::now()",
    )
    .bind(("rid", rid))
    .bind(("winner", winner))
    .await?;
    Ok(())
}

fn parse_room_id(s: &str) -> surrealdb::types::RecordId {
    surrealdb::types::RecordId::parse_simple(s)
        .unwrap_or_else(|_| surrealdb::types::RecordId::new("room", s))
}

pub fn spawn_turn_watcher<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    timeout: Duration,
    on_timeout: TimeoutCallback,
) -> tokio::task::JoinHandle<()> {
    let mut subscriber = room.subscribe();
    let room_id = room.room_id.clone();
    tokio::spawn(async move {
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
                            // Re-evaluate pending set after every committed event.
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
                            // Resync after lag.
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
                            // Room dropped; nothing to watch.
                            return;
                        }
                    }
                }
                _ = wait_until(deadline) => {
                    // Timer fired with the same pending set still in
                    // place. Mark them faulted and exit.
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

        // Advance virtual time past the timeout.
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

        // Half the window passes, alice acts, half-window again.
        tokio::time::advance(Duration::from_secs(3)).await;
        live.handle_act("alice", "MOVE", "").await.unwrap();
        // Pending now is just [bob] — timer must restart, not yet fire.
        tokio::time::advance(Duration::from_secs(3)).await;
        // Need to yield so the watcher task processes the broadcast.
        tokio::task::yield_now().await;
        assert_eq!(fires.load(Ordering::SeqCst), 0);

        // Push past the new window.
        tokio::time::advance(Duration::from_secs(5)).await;
        // Watcher fires + exits.
        tokio::time::sleep(Duration::from_millis(1)).await;
        assert_eq!(fires.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn timer_idle_when_pending_set_is_empty() {
        let storage = Storage::memory();
        let registry = Arc::new(RoomRegistry::<Pair>::new(storage.clone(), "j3".into()));
        let live = registry.open("r3", Duration::from_secs(60)).await.unwrap();
        // No START yet -> not playing -> no pending.
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
