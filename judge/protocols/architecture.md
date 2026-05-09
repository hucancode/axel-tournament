# Architecture

Three layers, three responsibilities, three boundaries.

```
┌──────────────────────────────────────────────────────────────────┐
│  Wire (src/protocol/mod.rs)         — parse / serialize frames   │
├──────────────────────────────────────────────────────────────────┤
│  RoomLogic + LiveRoom               — pure state + broadcast     │
│  (src/services/room_logic.rs)                                    │
├──────────────────────────────────────────────────────────────────┤
│  Storage (src/services/storage)     — lease + log + meta (IO)    │
│   ├─ LeaseStore                                                  │
│   ├─ EventLog                                                    │
│   └─ MetaIndex                                                   │
└──────────────────────────────────────────────────────────────────┘
```

The boundaries are strict:

- Wire knows nothing about state. It is pure parse / serialize.
- `RoomLogic` knows nothing about IO. `validate` and `fold` are pure
  functions; they take state and a `(kind, payload)` pair and produce
  either a `Vec<(kind, payload)>` of events to emit or an error string.
- Each storage trait owns one concern. `LeaseStore` brokers ownership.
  `EventLog` appends and reads. `MetaIndex` maintains the discovery
  view. The traits are independent so each can be mocked in isolation.

`LiveRoom` is the only place all three meet:

```
handle_act(player, kind, payload):
    lock state (write)
    events = L::validate(&state, player, kind, payload)?
    for (k, p) in events:
        e = log.append(room, owner, k, p).await?      // fence-checked
        L::fold(&mut state, &e.kind, &e.payload)
        head = e.seq
        broadcast(e)
    unlock
    meta.upsert(room, snapshot, head)                  // best effort
```

The write lock is held across the whole sequence so concurrent ACTs
see a consistent state.

## RoomLogic

```rust
pub trait RoomLogic: Send + Sync + 'static {
    type State: Default + Send + Sync + 'static;

    fn fold(state: &mut Self::State, kind: &str, payload: &str);
    fn validate(state: &Self::State, player: &str, kind: &str, payload: &str)
        -> Result<Vec<(String, String)>, String>;

    fn game_id() -> &'static str;
    fn max_players() -> usize;
    fn snapshot(state: &Self::State) -> RoomSnapshot;
}
```

- `fold` is total over events the log already accepted. It must not
  panic on malformed payloads — treat unrecognised input as a no-op
  rather than aborting, since the log is the source of truth and the
  process must replay it deterministically.
- `validate` decides whether a proposed action is legal. It returns the
  events to emit (possibly empty for a no-op) or an error string the
  server logs and silently drops. Wire-level errors never leak the
  message.
- `snapshot` projects state into a small struct used by `MetaIndex`. It
  is a view, not a source of truth.

The `kind` strings and what each one means live in the per-game
protocol document.

## Storage

Three traits, each independently mockable.

```rust
#[async_trait]
pub trait LeaseStore: Send + Sync + 'static {
    async fn acquire(&self, room: &str, owner: &str, ttl: Duration) -> Result<bool>;
    async fn renew  (&self, room: &str, owner: &str, ttl: Duration) -> Result<bool>;
    async fn release(&self, room: &str, owner: &str) -> Result<()>;
    async fn rooms_owned_by(&self, owner: &str) -> Result<Vec<String>>;
}

#[async_trait]
pub trait EventLog: Send + Sync + 'static {
    /// Append `(kind, payload)`. Fenced on lease ownership: fails
    /// atomically if the caller no longer owns the lease.
    async fn append(&self, room: &str, owner: &str, kind: &str, payload: &str)
        -> Result<Event>;
    async fn read_since(&self, room: &str, since: u64) -> Result<Vec<Event>>;
    async fn head      (&self, room: &str) -> Result<u64>;
}

#[async_trait]
pub trait MetaIndex: Send + Sync + 'static {
    async fn upsert(&self, room: &str, game_id: &str,
                    snapshot: &RoomSnapshot, head: u64) -> Result<()>;
    async fn list  (&self, game_id: Option<&str>, phase: Option<&str>,
                    limit: u32) -> Result<Vec<RoomMeta>>;
}
```

`EventLog::append` is the safety pivot: it consults the lease in the
same transaction and fails atomically if the caller is no longer the
owner. This is the only safety property the runtime relies on for
split-brain prevention; the rest is monotonicity of `seq`.

Two storage stacks ship:

- `SurrealStorage` — production. One struct that implements all three
  traits over `room_event` / `room_lease` / `room_meta` tables.
- `MemoryStorage` — tests. `Mutex<Inner>` with the same fence and
  monotonicity. No schemas, no migrations, no Tokio runtime apart
  from the trait's `async`.

A test exercising the full `validate → append → fold → broadcast`
pipeline constructs a `MemoryStorage` and a `LiveRoom<L>` and drives
`handle_act` directly. No database is required and no test setup is
global.

## LiveRoom runtime

`LiveRoom<L>` owns:

- the in-memory `L::State`, behind a `RwLock`,
- a `tokio::sync::broadcast::Sender<Event>` for live subscribers,
- handles to a `LeaseStore` and an `EventLog`.

Read paths (`head`, `read_since`) hit `EventLog` directly. Live
subscribers are fed by the broadcast channel; clients that fall behind
by more than the channel's capacity are disconnected so they reconnect
with `since_seq` and gap-fill cleanly. Discovery refreshes (via
`MetaIndex`) are best-effort and skipped when the snapshot is
unchanged.

## Lease and failover

- Each loaded room renews its lease every 5 s with a 15 s TTL.
- A judge that crashes loses its lease within ≤ 15 s. Any judge
  receiving a fresh `HELLO` for that room can acquire and rebuild
  state by replaying the log.
- Concurrent appends are impossible because `EventLog::append` checks
  the lease at the storage layer (fence). A judge whose lease silently
  expired will fail to append rather than diverge.

## Storage layout (SurrealDB)

```
room_event:  { id, room: string, seq: int, kind, payload, ts }
room_lease:  { id (= room_id), owner: string, expires: datetime }
room_meta:   { id (= room_id), game_id, phase, host, players, head, updated_at }
```

`seq` is assigned inside the append transaction (`max(seq) + 1`).
`room_meta` is a discovery index, refreshed by the lease holder after
every state-affecting append. It is rebuildable from the log at any
time.

## Testing recipe (no database)

Every protocol claim above is exercised by tests that need no DB:

1. **Wire round-trip** — unit tests in `src/protocol/mod.rs`.
2. **RoomLogic per game** — unit tests next to `*_logic.rs`.
3. **Pipeline** — `handle_act` against `MemoryStorage`. Lives both as
   unit tests in `src/services/room_logic.rs` and as integration
   tests in `judge/tests/protocol.rs`.
4. **Lease and fence** — `MemoryStorage` enforces the same fence as
   the SurrealDB impl, so split-brain tests run without a DB.

Anything that requires a real SurrealDB lives in `judge/tests/connection.rs`
and is exercised in CI against an in-memory SurrealDB container — but
no protocol test needs it.

## AI-vs-AI: same protocol, stdio transport

AI tournament matches run through the same pipeline. A
`services::match_watcher` claim becomes:

1. `RoomRegistry::open` to acquire a lease and spawn a `LiveRoom<L>`.
2. One `BotConn` per submission: a subprocess wired to the room.
   - Server writes `EVENT seq kind payload\n` lines to the bot's stdin.
   - Bot writes `ACT kind [payload]\n` lines to its stdout.
   - On `ACT`, the conn calls `LiveRoom::handle_act` exactly like the
     WebSocket handler does.
3. The conn tears the bot down on `WINNER` / `DRAW` / `GAME_END`.

Bots have no `HELLO`, no `since_seq`, no reconnect. They are
short-lived and pre-authorized. If a bot crashes mid-match the match
fails — there is nothing to recover. The lease + log machinery is
unchanged: a different judge taking over the room would respawn the
bots from the submissions, replaying the log on the way up.

`RoomLogic`, `EventLog`, the seven wire frames, and the per-game
`kind` strings are all the same as for human rooms. A submission that
plays a bot match behaves identically to a manual WebSocket client
connected to the same room.

The legacy `services::match_watcher` + per-game `Game::run` runners
predate this design. They are slated for removal — see the migration
note in `judge/src/services/match_watcher.rs`.
