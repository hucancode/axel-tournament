// Full-pipeline integration tests against real SurrealDB storage.
//
// Connection settings come from `Config::from_env()` — no hard-coded
// URLs, namespaces, or credentials. Tests assume the DB is already
// running (see `make test-db-up`). Each test uses unique room IDs and
// owner IDs so concurrent runs never collide.
//
// Spec mirrored: judge/tests/protocol.rs (which runs the same shapes
// against MemoryStorage). If a scenario is added here, add the matching
// in-memory test there too — the storage trait is the contract.

mod db;

use std::sync::Arc;
use std::time::Duration;

use judge::games::{Pd, Rps, Ttt};
use judge::services::room::logic::{LiveRoom, RoomLogic, RoomRegistry};
use judge::services::storage::{Event, Storage};

const LEASE: Duration = Duration::from_secs(15);

fn unique(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

async fn storage() -> Storage {
    Storage::surreal(db::setup_test_db().await)
}

/// Drive the live room and collect every event the broadcast emitted
/// during the closure. Subscribed before the closure starts.
async fn collect_events<L, F, Fut>(live: &Arc<LiveRoom<L>>, body: F) -> Vec<Event>
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

// ---------- Lease ----------

#[tokio::test]
async fn lease_blocks_then_releases() {
    let s = storage().await;
    let room = unique("room");
    let a = unique("judge-A");
    let b = unique("judge-B");

    assert!(s.lease.acquire(&room, &a, LEASE).await.unwrap());
    assert!(!s.lease.acquire(&room, &b, LEASE).await.unwrap());

    s.lease.release(&room, &a).await.unwrap();
    assert!(s.lease.acquire(&room, &b, LEASE).await.unwrap());
    s.lease.release(&room, &b).await.unwrap();
}

#[tokio::test]
async fn lease_takeover_after_expiry() {
    let s = storage().await;
    let room = unique("room");
    let a = unique("judge-A");
    let b = unique("judge-B");

    s.lease.acquire(&room, &a, Duration::from_secs(1)).await.unwrap();
    assert!(!s.lease.acquire(&room, &b, LEASE).await.unwrap());

    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(s.lease.acquire(&room, &b, LEASE).await.unwrap());

    let res = s.log.append(&room, &a, "JOIN", "x").await;
    assert!(res.is_err(), "stale owner append must fail (fence)");
    s.lease.release(&room, &b).await.unwrap();
}

#[tokio::test]
async fn lease_renew_extends_ownership() {
    let s = storage().await;
    let room = unique("room");
    let a = unique("judge-A");

    s.lease.acquire(&room, &a, Duration::from_secs(1)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert!(s.lease.renew(&room, &a, Duration::from_secs(5)).await.unwrap());
    tokio::time::sleep(Duration::from_secs(2)).await;
    // Original 1s window has passed; renew kept it alive.
    assert!(s.log.append(&room, &a, "PLAYER_JOINED", "p").await.is_ok());
    s.lease.release(&room, &a).await.unwrap();
}

#[tokio::test]
async fn rooms_owned_by_lists_only_my_leases() {
    let s = storage().await;
    let owner = unique("judge");
    let other = unique("other");
    let r1 = unique("room");
    let r2 = unique("room");
    let r3 = unique("room");

    s.lease.acquire(&r1, &owner, LEASE).await.unwrap();
    s.lease.acquire(&r2, &owner, LEASE).await.unwrap();
    s.lease.acquire(&r3, &other, LEASE).await.unwrap();

    let mine = s.lease.rooms_owned_by(&owner).await.unwrap();
    assert!(mine.contains(&r1));
    assert!(mine.contains(&r2));
    assert!(!mine.contains(&r3));

    s.lease.release(&r1, &owner).await.unwrap();
    s.lease.release(&r2, &owner).await.unwrap();
    s.lease.release(&r3, &other).await.unwrap();
}

// ---------- Append + fence ----------

#[tokio::test]
async fn append_assigns_monotonic_seq_and_replays() {
    let s = storage().await;
    let room = unique("room");
    let owner = unique("judge");

    s.lease.acquire(&room, &owner, LEASE).await.unwrap();

    let e1 = s.log.append(&room, &owner, "PLAYER_JOINED", "alice").await.unwrap();
    let e2 = s.log.append(&room, &owner, "PLAYER_JOINED", "bob").await.unwrap();
    let e3 = s.log.append(&room, &owner, "GAME_STARTED", "5").await.unwrap();
    assert_eq!((e1.seq, e2.seq, e3.seq), (1, 2, 3));

    let all = s.log.read_since(&room, 0).await.unwrap();
    assert_eq!(all.len(), 3);
    let tail = s.log.read_since(&room, 2).await.unwrap();
    assert_eq!(tail.len(), 1);
    assert_eq!(tail[0].seq, 3);
    assert_eq!(s.log.head(&room).await.unwrap(), 3);

    s.lease.release(&room, &owner).await.unwrap();
}

#[tokio::test]
async fn append_without_lease_fails() {
    let s = storage().await;
    let room = unique("room");
    let rogue = unique("rogue");
    let res = s.log.append(&room, &rogue, "JOIN", "x").await;
    assert!(res.is_err(), "append without lease must fail");
}

// ---------- RPS pipeline ----------

#[tokio::test]
async fn rps_full_match_emits_documented_events() {
    let s = storage().await;
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();

    let alice = unique("user");
    let bob = unique("user");

    let events = collect_events(&live, |live| {
        let alice = alice.clone();
        let bob = bob.clone();
        async move {
            live.handle_act(&alice, "JOIN", "").await.unwrap();
            live.handle_act(&bob, "JOIN", "").await.unwrap();
            live.handle_act(&alice, "START", "").await.unwrap();
            live.handle_act(&alice, "MOVE", "ROCK").await.unwrap();
            live.handle_act(&bob, "MOVE", "SCISSORS").await.unwrap();
        }
    })
    .await;

    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec!["PLAYER_JOINED", "PLAYER_JOINED", "GAME_STARTED",
             "MOVE", "MOVE", "ROUND_RESULT"]
    );
    let seqs: Vec<u64> = events.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, (1..=6).collect::<Vec<_>>());

    let rr = events.iter().find(|e| e.kind == "ROUND_RESULT").unwrap();
    assert!(rr.payload.starts_with("1 ROCK SCISSORS "));
    assert!(rr.payload.ends_with(" 1 0"));

    registry.drop_room(&room).await;
}

#[tokio::test]
async fn rps_silent_drop_on_invalid_action() {
    let s = storage().await;
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();
    let alice = unique("user");

    live.handle_act(&alice, "JOIN", "").await.unwrap();
    let head_before = live.head();
    let r = live.handle_act(&alice, "MOVE", "ROCK").await;
    assert!(r.is_err());
    assert_eq!(live.head(), head_before, "rejected action must not append");

    registry.drop_room(&room).await;
}

// ---------- Tic-tac-toe pipeline ----------

#[tokio::test]
async fn ttt_x_wins_top_row() {
    let s = storage().await;
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Ttt>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();
    let x = unique("user");
    let o = unique("user");

    let events = collect_events(&live, |live| {
        let x = x.clone();
        let o = o.clone();
        async move {
            live.handle_act(&x, "JOIN", "").await.unwrap();
            live.handle_act(&o, "JOIN", "").await.unwrap();
            // 3x3 board with 3-in-a-row to win (classic tic-tac-toe).
            live.handle_act(&x, "START", "3 3").await.unwrap();
            live.handle_act(&x, "MOVE", "0 0").await.unwrap();
            live.handle_act(&o, "MOVE", "1 0").await.unwrap();
            live.handle_act(&x, "MOVE", "0 1").await.unwrap();
            live.handle_act(&o, "MOVE", "1 1").await.unwrap();
            live.handle_act(&x, "MOVE", "0 2").await.unwrap();
        }
    })
    .await;

    let last = events.last().unwrap();
    assert_eq!(last.kind, "WINNER");
    assert_eq!(last.payload, "0");

    registry.drop_room(&room).await;
}

#[tokio::test]
async fn ttt_off_turn_move_is_silent() {
    let s = storage().await;
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Ttt>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();
    let x = unique("user");
    let o = unique("user");

    live.handle_act(&x, "JOIN", "").await.unwrap();
    live.handle_act(&o, "JOIN", "").await.unwrap();
    live.handle_act(&x, "START", "").await.unwrap();
    let head_before = live.head();
    let r = live.handle_act(&o, "MOVE", "0 0").await;
    assert!(r.is_err());
    assert_eq!(live.head(), head_before);

    registry.drop_room(&room).await;
}

// ---------- Prisoner's dilemma pipeline ----------

#[tokio::test]
async fn pd_round_result_uses_payoff_matrix() {
    let s = storage().await;
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Pd>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();
    let a = unique("user");
    let b = unique("user");

    live.handle_act(&a, "JOIN", "").await.unwrap();
    live.handle_act(&b, "JOIN", "").await.unwrap();
    live.handle_act(&a, "START", "").await.unwrap();

    let events = collect_events(&live, |live| {
        let a = a.clone();
        let b = b.clone();
        async move {
            live.handle_act(&a, "MOVE", "C").await.unwrap();
            live.handle_act(&b, "MOVE", "D").await.unwrap();
        }
    })
    .await;

    let rr = events.iter().find(|e| e.kind == "ROUND_RESULT").unwrap();
    assert!(rr.payload.starts_with("1 C D "));
    assert!(rr.payload.ends_with(" 0 5"));

    registry.drop_room(&room).await;
}

// ---------- Reconnect / replay ----------

#[tokio::test]
async fn reconnect_gap_fills_from_log() {
    let s = storage().await;
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();
    let a = unique("user");
    let b = unique("user");

    live.handle_act(&a, "JOIN", "").await.unwrap();
    live.handle_act(&b, "JOIN", "").await.unwrap();
    live.handle_act(&a, "START", "").await.unwrap();

    assert_eq!(live.head(), 3);
    let gap = live.read_since(1).await.unwrap();
    assert_eq!(gap.len(), 2);
    assert_eq!(gap[0].seq, 2);
    assert_eq!(gap[1].seq, 3);
    assert_eq!(gap[1].kind, "GAME_STARTED");

    registry.drop_room(&room).await;
}

#[tokio::test]
async fn replay_after_handoff_reconstructs_state() {
    let s = storage().await;
    let room = unique("room");
    let a = unique("user");
    let b = unique("user");

    let r1 = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge-A")));
    let live = r1.open(&room, LEASE).await.unwrap();
    live.handle_act(&a, "JOIN", "").await.unwrap();
    live.handle_act(&b, "JOIN", "").await.unwrap();
    live.handle_act(&a, "START", "").await.unwrap();
    drop(live);
    r1.drop_room(&room).await;

    let r2 = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge-B")));
    let live = r2.open(&room, LEASE).await.unwrap();
    assert_eq!(live.head(), 3);
    let (players, host) = live.with_state(|st| (st.players.clone(), st.host.clone())).await;
    assert_eq!(players, vec![a.clone(), b]);
    assert_eq!(host.as_deref(), Some(a.as_str()));

    r2.drop_room(&room).await;
}

#[tokio::test]
async fn registry_blocks_other_judge() {
    let s = storage().await;
    let room = unique("room");
    let r1 = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge-A")));
    r1.open(&room, LEASE).await.unwrap();

    let r2 = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge-B")));
    let res = r2.open(&room, LEASE).await;
    assert!(res.is_err(), "second judge must be locked out");

    r1.drop_room(&room).await;
}

// ---------- Discovery (MetaIndex) ----------

#[tokio::test]
async fn discovery_lists_rooms_and_filters_phase() {
    let s = storage().await;
    let game = Rps::game_id();
    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge")));

    let lobby_room = unique("room");
    let playing_room = unique("room");
    let alice = unique("user");
    let bob = unique("user");

    let lobby = registry.open(&lobby_room, LEASE).await.unwrap();
    lobby.handle_act(&alice, "JOIN", "").await.unwrap();

    let playing = registry.open(&playing_room, LEASE).await.unwrap();
    playing.handle_act(&alice, "JOIN", "").await.unwrap();
    playing.handle_act(&bob, "JOIN", "").await.unwrap();
    playing.handle_act(&alice, "START", "").await.unwrap();

    let listing = s.meta.list(Some(game), None, 500).await.unwrap();
    assert!(listing.iter().any(|m| m.id == lobby_room));
    assert!(listing.iter().any(|m| m.id == playing_room));

    let lobbies = s.meta.list(Some(game), Some("lobby"), 500).await.unwrap();
    assert!(lobbies.iter().any(|m| m.id == lobby_room));
    assert!(!lobbies.iter().any(|m| m.id == playing_room));

    let playings = s.meta.list(Some(game), Some("playing"), 500).await.unwrap();
    assert!(playings.iter().any(|m| m.id == playing_room));
    assert!(!playings.iter().any(|m| m.id == lobby_room));

    registry.drop_room(&lobby_room).await;
    registry.drop_room(&playing_room).await;
}

#[tokio::test]
async fn discovery_skips_meta_upsert_for_chat_only() {
    let s = storage().await;
    let game = Rps::game_id();
    let room = unique("room");
    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), unique("judge")));
    let live = registry.open(&room, LEASE).await.unwrap();
    let alice = unique("user");

    live.handle_act(&alice, "JOIN", "").await.unwrap();
    let snap_after_join = s.meta.list(Some(game), None, 500).await.unwrap();
    let head_after_join = snap_after_join
        .iter()
        .find(|m| m.id == room)
        .unwrap()
        .head;

    live.handle_act(&alice, "CHAT", "hi").await.unwrap();
    let snap_after_chat = s.meta.list(Some(game), None, 500).await.unwrap();
    let row = snap_after_chat.iter().find(|m| m.id == room).unwrap();
    assert_eq!(row.head, head_after_join, "CHAT keeps snapshot, skips upsert");

    registry.drop_room(&room).await;
}

// ---------- Auth ----------

#[tokio::test]
async fn jwt_round_trip_against_env_secret() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use judge::middleware::auth::Claims;

    let secret = judge::Config::from_env().jwt_secret;
    let user_id = unique("user");
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.clone(),
        email: "t@example.com".into(),
        role: axel_core::models::user::UserRole::Player,
        exp: now + 3600,
        iat: now,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    let parsed = axel_core::auth::validate_token(&secret, &token).unwrap();
    assert_eq!(parsed.sub, user_id);
    assert!(axel_core::auth::validate_token(&secret, "not.a.token").is_err());
    assert!(axel_core::auth::validate_token("wrong-secret", &token).is_err());
}
