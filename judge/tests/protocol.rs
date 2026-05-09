// Protocol integration tests. No database, no real network.
//
// These tests prove the architecture's testability claim: the wire +
// RoomLogic + LiveRoom pipeline runs end-to-end against `MemoryStorage`,
// with no migrations, no DB container, and no global state.
//
// Spec: judge/protocols/architecture.md, judge/protocols/wire.md.

use std::sync::Arc;
use std::time::Duration;

use judge::games::{Pd, Rps, Ttt};
use judge::protocol::{parse_client, parse_server, serialize_client, serialize_server,
                      ClientFrame, ServerFrame};
use judge::services::room_logic::{LiveRoom, RoomLogic, RoomRegistry};
use judge::services::storage::Storage;

const LEASE: Duration = Duration::from_secs(60);

// ---------- helpers ----------

async fn open_room<L: RoomLogic>(owner: &str, room: &str) -> (Storage, Arc<LiveRoom<L>>) {
    let storage = Storage::memory();
    let registry = Arc::new(RoomRegistry::<L>::new(storage.clone(), owner.into()));
    let live = registry.open(room, LEASE).await.unwrap();
    (storage, live)
}

/// Drive the live room and collect all events broadcast during a closure.
async fn collect_events<L, F, Fut>(live: &Arc<LiveRoom<L>>, body: F) -> Vec<judge::services::storage::Event>
where
    L: RoomLogic,
    F: FnOnce(Arc<LiveRoom<L>>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut sub = live.subscribe();
    body(live.clone()).await;
    let mut out = Vec::new();
    while let Ok(ev) = sub.try_recv() {
        out.push(ev);
    }
    out
}

// ---------- wire-layer round trips (cross-checked with the docs) ----------

#[test]
fn wire_client_round_trip() {
    let cases = [
        ClientFrame::Hello { jwt: "abc.def".into(), since: 0 },
        ClientFrame::Hello { jwt: "tok".into(), since: 17 },
        ClientFrame::Act { kind: "JOIN".into(), payload: "".into() },
        ClientFrame::Act { kind: "MOVE".into(), payload: "1 2".into() },
        ClientFrame::Act { kind: "CHAT".into(), payload: "héllo wörld".into() },
        ClientFrame::Pong,
    ];
    for f in cases {
        let line = serialize_client(&f);
        let back = parse_client(&line).unwrap();
        assert_eq!(back, f);
        assert!(!line.contains('\n'), "frame must be single-line: {line:?}");
    }
}

#[test]
fn wire_server_round_trip() {
    let cases = [
        ServerFrame::Welcome { player_id: "user:alice".into(), head: 42 },
        ServerFrame::Event { seq: 1, kind: "GAME_STARTED".into(), payload: "".into() },
        ServerFrame::Event { seq: 2, kind: "MOVE".into(), payload: "user:alice ROCK".into() },
        ServerFrame::Err { code: "AUTH".into(), msg: "bad token".into() },
        ServerFrame::Ping,
    ];
    for f in cases {
        let line = serialize_server(&f);
        let back = parse_server(&line).unwrap();
        assert_eq!(back, f);
    }
}

// ---------- RPS pipeline ----------

#[tokio::test]
async fn rps_full_match_emits_documented_events() {
    let (_s, live) = open_room::<Rps>("judge-A", "r").await;

    let events = collect_events(&live, |live| async move {
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "").await.unwrap();
        live.handle_act("alice", "MOVE", "ROCK").await.unwrap();
        live.handle_act("bob", "MOVE", "SCISSORS").await.unwrap();
    })
    .await;

    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec!["PLAYER_JOINED", "PLAYER_JOINED", "GAME_STARTED",
             "MOVE", "MOVE", "ROUND_RESULT"]
    );

    // seq must be strictly monotonic.
    let seqs: Vec<u64> = events.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, (1..=6).collect::<Vec<_>>());

    // ROUND_RESULT payload format: <round> <m0> <m1> <s0> <s1>.
    let rr = events.iter().find(|e| e.kind == "ROUND_RESULT").unwrap();
    assert!(rr.payload.starts_with("1 ROCK SCISSORS "));
    assert!(rr.payload.ends_with(" 1 0"));
}

#[tokio::test]
async fn rps_silent_drop_on_invalid_action() {
    let (_s, live) = open_room::<Rps>("judge-A", "r").await;
    live.handle_act("alice", "JOIN", "").await.unwrap();

    // Move while still in lobby — wire spec says validate failures are
    // silent: the API call returns Err but no event is appended.
    let result = live.handle_act("alice", "MOVE", "ROCK").await;
    assert!(result.is_err());
    assert_eq!(live.head(), 1, "no append should have happened");
}

#[tokio::test]
async fn rps_chat_does_not_change_phase_snapshot() {
    let (storage, live) = open_room::<Rps>("judge-A", "r").await;
    live.handle_act("alice", "JOIN", "").await.unwrap();
    let head_before = live.head();
    live.handle_act("alice", "CHAT", "hi").await.unwrap();
    assert_eq!(live.head(), head_before + 1);

    // MetaIndex was upserted at least once on JOIN; CHAT skips it (snapshot
    // unchanged), so the listing still shows lobby + alice.
    let listing = storage.meta.list(Some("rock-paper-scissors"), None, 10).await.unwrap();
    assert_eq!(listing.len(), 1);
    assert_eq!(listing[0].phase, "lobby");
    assert_eq!(listing[0].players, vec!["alice"]);
}

// ---------- Tic-tac-toe pipeline ----------

#[tokio::test]
async fn ttt_x_wins_top_row() {
    let (_s, live) = open_room::<Ttt>("judge-A", "r").await;

    let events = collect_events(&live, |live| async move {
        live.handle_act("x", "JOIN", "").await.unwrap();
        live.handle_act("o", "JOIN", "").await.unwrap();
        live.handle_act("x", "START", "").await.unwrap();
        live.handle_act("x", "MOVE", "0 0").await.unwrap();
        live.handle_act("o", "MOVE", "1 0").await.unwrap();
        live.handle_act("x", "MOVE", "0 1").await.unwrap();
        live.handle_act("o", "MOVE", "1 1").await.unwrap();
        live.handle_act("x", "MOVE", "0 2").await.unwrap();
    })
    .await;

    let last = events.last().unwrap();
    assert_eq!(last.kind, "WINNER");
    assert_eq!(last.payload, "0", "player 0 (x) wins");
}

#[tokio::test]
async fn ttt_off_turn_move_is_silent() {
    let (_s, live) = open_room::<Ttt>("judge-A", "r").await;
    live.handle_act("x", "JOIN", "").await.unwrap();
    live.handle_act("o", "JOIN", "").await.unwrap();
    live.handle_act("x", "START", "").await.unwrap();
    let head_before = live.head();
    let r = live.handle_act("o", "MOVE", "0 0").await;
    assert!(r.is_err());
    assert_eq!(live.head(), head_before);
}

// ---------- Prisoner's dilemma pipeline ----------

#[tokio::test]
async fn pd_round_result_uses_payoff_matrix() {
    let (_s, live) = open_room::<Pd>("judge-A", "r").await;

    live.handle_act("a", "JOIN", "").await.unwrap();
    live.handle_act("b", "JOIN", "").await.unwrap();
    live.handle_act("a", "START", "").await.unwrap();

    let events = collect_events(&live, |live| async move {
        live.handle_act("a", "MOVE", "C").await.unwrap();
        live.handle_act("b", "MOVE", "D").await.unwrap();
    })
    .await;

    let rr = events.iter().find(|e| e.kind == "ROUND_RESULT").unwrap();
    // payoff matrix says (C, D) = (0, 5) for player 0 / player 1.
    assert!(rr.payload.starts_with("1 C D "));
    assert!(rr.payload.ends_with(" 0 5"));
}

// ---------- Reconnect / gap-fill ----------

#[tokio::test]
async fn reconnect_gap_fills_from_log() {
    let (_s, live) = open_room::<Rps>("judge-A", "r").await;

    live.handle_act("alice", "JOIN", "").await.unwrap();
    live.handle_act("bob", "JOIN", "").await.unwrap();
    live.handle_act("alice", "START", "").await.unwrap();

    // Imagine a client that disconnected after seeing seq 1.
    let head = live.head();
    assert_eq!(head, 3);

    let gap = live.read_since(1).await.unwrap();
    assert_eq!(gap.len(), 2);
    assert_eq!(gap[0].seq, 2);
    assert_eq!(gap[0].kind, "PLAYER_JOINED");
    assert_eq!(gap[1].seq, 3);
    assert_eq!(gap[1].kind, "GAME_STARTED");
}

// ---------- Lease and failover ----------

#[tokio::test]
async fn lease_takeover_requires_expiry() {
    let storage = Storage::memory();

    // Judge A holds the lease.
    let r1 = Arc::new(RoomRegistry::<Rps>::new(storage.clone(), "judge-A".into()));
    r1.open("r", LEASE).await.unwrap();

    // Judge B is locked out while A's lease is fresh.
    let r2 = Arc::new(RoomRegistry::<Rps>::new(storage.clone(), "judge-B".into()));
    let blocked = r2.open("r", LEASE).await;
    assert!(blocked.is_err());
}

#[tokio::test]
async fn replay_after_handoff() {
    let storage = Storage::memory();

    // Judge A drives a partial match, then drops the room (lease released).
    let r1 = Arc::new(RoomRegistry::<Rps>::new(storage.clone(), "judge-A".into()));
    let live = r1.open("r", LEASE).await.unwrap();
    live.handle_act("alice", "JOIN", "").await.unwrap();
    live.handle_act("bob", "JOIN", "").await.unwrap();
    live.handle_act("alice", "START", "").await.unwrap();
    drop(live);
    r1.drop_room("r").await;

    // Judge B picks up; replay must reconstruct state from the log alone.
    let r2 = Arc::new(RoomRegistry::<Rps>::new(storage, "judge-B".into()));
    let live = r2.open("r", LEASE).await.unwrap();
    assert_eq!(live.head(), 3);
    let (players, host) =
        live.with_state(|st| (st.players.clone(), st.host.clone())).await;
    assert_eq!(players, vec!["alice", "bob"]);
    assert_eq!(host.as_deref(), Some("alice"));
}

// ---------- Discovery (MetaIndex) ----------

#[tokio::test]
async fn discovery_lists_loaded_rooms() {
    let storage = Storage::memory();
    let r = Arc::new(RoomRegistry::<Rps>::new(storage.clone(), "judge".into()));

    let a = r.open("a", LEASE).await.unwrap();
    let b = r.open("b", LEASE).await.unwrap();
    a.handle_act("alice", "JOIN", "").await.unwrap();
    b.handle_act("bob", "JOIN", "").await.unwrap();
    b.handle_act("carol", "JOIN", "").await.unwrap();
    b.handle_act("bob", "START", "").await.unwrap();

    let listing = storage.meta.list(Some("rock-paper-scissors"), None, 10).await.unwrap();
    assert_eq!(listing.len(), 2);

    let lobbies = storage.meta.list(Some("rock-paper-scissors"), Some("lobby"), 10).await.unwrap();
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies[0].id, "a");

    let playing = storage.meta.list(Some("rock-paper-scissors"), Some("playing"), 10).await.unwrap();
    assert_eq!(playing.len(), 1);
    assert_eq!(playing[0].id, "b");
    assert_eq!(playing[0].players, vec!["bob", "carol"]);
}

// ---------- Wire + RoomLogic loop ----------
//
// The contract is: the server appends events whose `(kind, payload)` is
// exactly what the wire `EVENT` frame carries. So a client can serialize
// a `WELCOME` + `EVENT*` stream from the live broadcast and parse it
// back with the wire parser, lossless.

#[tokio::test]
async fn server_events_serialize_and_parse_back() {
    let (_s, live) = open_room::<Rps>("judge-A", "r").await;

    let events = collect_events(&live, |live| async move {
        live.handle_act("alice", "JOIN", "").await.unwrap();
        live.handle_act("bob", "JOIN", "").await.unwrap();
        live.handle_act("alice", "START", "").await.unwrap();
        live.handle_act("alice", "MOVE", "ROCK").await.unwrap();
        live.handle_act("bob", "MOVE", "PAPER").await.unwrap();
    })
    .await;

    for e in events {
        let frame = ServerFrame::Event {
            seq: e.seq,
            kind: e.kind.clone(),
            payload: e.payload.clone(),
        };
        let line = serialize_server(&frame);
        let back = parse_server(&line).unwrap();
        assert_eq!(back, frame);
    }
}
