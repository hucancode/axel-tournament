// Room logic + live runner. Spec: judge/protocols/architecture.md.
//
// `RoomLogic` is the per-game contract (pure `fold` + `validate`).
// `LiveRoom<L>` is the runtime: validate -> append -> fold -> broadcast,
// with the write lock held across the sequence so concurrent ACTs see
// a consistent state.

mod contract;
mod live;
mod registry;

pub use contract::RoomLogic;
pub use live::LiveRoom;
pub use registry::{OnRoomOpened, RoomRegistry};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::storage::{RoomSnapshot, Storage};
    use std::sync::Arc;
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
