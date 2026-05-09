// Integration tests against the SurrealDB-backed storage stack.
// DB-free protocol tests live in `judge/tests/protocol.rs` and in the
// unit tests under `judge/src/services/`.

mod db;

use std::sync::Arc;
use std::time::Duration;

use judge::games::Rps;
use judge::services::room_logic::RoomRegistry;
use judge::services::storage::Storage;

fn unique_room_id() -> String {
    format!("room_{}", uuid::Uuid::new_v4().simple())
}

async fn storage() -> Storage {
    let db = db::setup_test_db().await;
    Storage::surreal(db)
}

#[tokio::test]
async fn lease_acquire_then_other_owner_blocked() {
    let s = storage().await;
    let room = unique_room_id();

    let ok = s.lease.acquire(&room, "judge-A", Duration::from_secs(15)).await.unwrap();
    assert!(ok, "judge-A should acquire fresh lease");

    let blocked = s.lease.acquire(&room, "judge-B", Duration::from_secs(15)).await.unwrap();
    assert!(!blocked, "judge-B must be blocked while judge-A holds lease");

    s.lease.release(&room, "judge-A").await.unwrap();
    let after_release = s.lease.acquire(&room, "judge-B", Duration::from_secs(15)).await.unwrap();
    assert!(after_release, "judge-B should acquire after release");
    s.lease.release(&room, "judge-B").await.unwrap();
}

#[tokio::test]
async fn append_assigns_monotonic_seq_and_replays() {
    let s = storage().await;
    let room = unique_room_id();

    s.lease.acquire(&room, "judge-A", Duration::from_secs(15)).await.unwrap();

    let e1 = s.log.append(&room, "judge-A", "PLAYER_JOINED", "alice").await.unwrap();
    let e2 = s.log.append(&room, "judge-A", "PLAYER_JOINED", "bob").await.unwrap();
    let e3 = s.log.append(&room, "judge-A", "GAME_STARTED", "5").await.unwrap();

    assert_eq!(e1.seq, 1);
    assert_eq!(e2.seq, 2);
    assert_eq!(e3.seq, 3);

    let all = s.log.read_since(&room, 0).await.unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].kind, "PLAYER_JOINED");
    assert_eq!(all[2].payload, "5");

    let tail = s.log.read_since(&room, 2).await.unwrap();
    assert_eq!(tail.len(), 1);
    assert_eq!(tail[0].seq, 3);

    s.lease.release(&room, "judge-A").await.unwrap();
}

#[tokio::test]
async fn append_without_lease_fails() {
    let s = storage().await;
    let room = unique_room_id();

    let res = s.log.append(&room, "judge-rogue", "JOIN", "x").await;
    assert!(res.is_err(), "append without lease must fail");
}

#[tokio::test]
async fn registry_open_loads_state_from_log() {
    let s = storage().await;
    let room = unique_room_id();

    s.lease.acquire(&room, "seeder", Duration::from_secs(15)).await.unwrap();
    s.log.append(&room, "seeder", "PLAYER_JOINED", "alice").await.unwrap();
    s.log.append(&room, "seeder", "PLAYER_JOINED", "bob").await.unwrap();
    s.log.append(&room, "seeder", "GAME_STARTED", "5").await.unwrap();
    s.lease.release(&room, "seeder").await.unwrap();

    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), "judge-loader".into()));
    let live = registry.open(&room, Duration::from_secs(15)).await.unwrap();

    assert_eq!(live.head(), 3);
    let (players, host, phase_is_playing) = live.with_state(|st| {
        use judge::games::rps_logic::Phase;
        (st.players.clone(), st.host.clone(), st.phase == Phase::Playing)
    }).await;
    assert_eq!(players, vec!["alice", "bob"]);
    assert_eq!(host.as_deref(), Some("alice"));
    assert!(phase_is_playing);

    registry.drop_room(&room).await;
}

#[tokio::test]
async fn handle_act_appends_and_folds() {
    let s = storage().await;
    let room = unique_room_id();

    let registry = Arc::new(RoomRegistry::<Rps>::new(s.clone(), "judge-act".into()));
    let live = registry.open(&room, Duration::from_secs(15)).await.unwrap();

    live.handle_act("alice", "JOIN", "").await.unwrap();
    live.handle_act("bob", "JOIN", "").await.unwrap();
    live.handle_act("alice", "START", "").await.unwrap();
    live.handle_act("alice", "MOVE", "ROCK").await.unwrap();
    live.handle_act("bob", "MOVE", "SCISSORS").await.unwrap();

    let head = live.head();
    assert!(head >= 5, "expected at least 5 events, got {head}");

    let events = s.log.read_since(&room, 0).await.unwrap();
    assert!(events.iter().any(|e| e.kind == "GAME_STARTED"));
    assert!(events.iter().any(|e| e.kind == "ROUND_RESULT"));

    let (scores, players) = live.with_state(|st| (st.scores, st.players.clone())).await;
    assert_eq!(scores, [1, 0]);
    assert_eq!(players, vec!["alice", "bob"]);

    registry.drop_room(&room).await;
}

#[tokio::test]
async fn lease_takeover_after_expiry() {
    let s = storage().await;
    let room = unique_room_id();

    s.lease.acquire(&room, "judge-A", Duration::from_secs(1)).await.unwrap();
    let blocked = s.lease.acquire(&room, "judge-B", Duration::from_secs(15)).await.unwrap();
    assert!(!blocked);

    tokio::time::sleep(Duration::from_secs(2)).await;
    let took_over = s.lease.acquire(&room, "judge-B", Duration::from_secs(15)).await.unwrap();
    assert!(took_over, "judge-B should take over expired lease");

    let res = s.log.append(&room, "judge-A", "JOIN", "x").await;
    assert!(res.is_err(), "stale judge-A append must fail");

    s.lease.release(&room, "judge-B").await.unwrap();
}

#[tokio::test]
async fn jwt_validation() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use judge::middleware::auth::{validate_jwt, Claims};

    let secret = "test_secret_key";
    let user_id = "user:test123";
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ).unwrap();

    let result = validate_jwt(&token, secret);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().sub, user_id);

    assert!(validate_jwt("invalid.token.here", secret).is_err());
    assert!(validate_jwt(&token, "wrong_secret").is_err());
}
