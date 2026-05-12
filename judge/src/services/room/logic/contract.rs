use crate::services::storage::RoomSnapshot;

/// Per-game logic. Pure: no I/O, no awaits.
pub trait RoomLogic: Send + Sync + 'static {
    type State: Default + Send + Sync + 'static;

    /// Apply an event already accepted by the log. Total over committed
    /// events: malformed payloads must degrade to a no-op rather than
    /// panicking, since replay must be deterministic.
    fn fold(state: &mut Self::State, kind: &str, payload: &str);

    /// Validate a proposed action. Returns the events to emit (possibly
    /// empty for a no-op) or an error string. Errors are dropped silently
    /// on the wire — see protocols/wire.md.
    fn validate(
        state: &Self::State,
        player_id: &str,
        kind: &str,
        payload: &str,
    ) -> Result<Vec<(String, String)>, String>;

    fn max_players() -> usize;
    fn game_id() -> &'static str;
    fn snapshot(state: &Self::State) -> RoomSnapshot;

    /// Players whose action is currently expected. Empty when nobody is
    /// blocking progress (lobby phase, between rounds, finished). The
    /// turn-timer watcher uses this to decide who to mark faulted on a
    /// timeout. Default: empty (turn enforcement disabled).
    fn pending_players(_state: &Self::State) -> Vec<String> {
        Vec::new()
    }

    /// Per-player time-pool budget in seconds, applied while the player
    /// is in `pending_players()`. Returns `None` when the game has no
    /// pool-based time control (the default), or when the host hasn't
    /// configured one. Same budget for every player — chess clocks are
    /// symmetric.
    fn time_pool_seconds(_state: &Self::State) -> Option<u64> {
        None
    }

    /// Per-turn timeout override in seconds. Returns `Some` when the
    /// host configured a strict per-move clock for this room. The turn
    /// watcher prefers this over the game-wide default from
    /// GameMetadata. `None` means "use the metadata default."
    fn per_turn_seconds(_state: &Self::State) -> Option<u64> {
        None
    }
}
