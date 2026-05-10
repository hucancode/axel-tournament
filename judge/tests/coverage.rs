// Service-level integration tests filling judge coverage gaps.
//
// Existing test files cover the heavy paths (compiler, bots, room
// protocol, full pipeline). This file fills the remaining surface so
// every public function in the judge crate has at least one direct
// test, and every documented edge case is exercised.
//
// Connection settings come from `judge::Config::from_env()` — no
// hard-coded URLs or credentials. Tests assume SurrealDB is running
// (`make test-db-up`). Each test uses unique room/match IDs so
// concurrent runs cannot collide.

mod db;

use judge::{
    games::{find_game_by_id, Pd, Rps, Ttt},
    middleware::auth::{validate_jwt, Claims},
    models::game_metadata::GAMES,
    services::{
        capacity::CapacityTracker,
        room::logic::RoomLogic,
        storage::Storage,
    },
    Config,
};

fn unique(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

// ---------- Config ----------

#[tokio::test]
async fn config_from_env_provides_defaults() {
    let cfg = Config::from_env();
    assert!(!cfg.database_url.is_empty());
    assert!(!cfg.database_ns.is_empty());
    assert!(!cfg.database_db.is_empty());
    assert!(!cfg.jwt_secret.is_empty());
    assert!(cfg.max_capacity > 0);
}

// ---------- CapacityTracker — gaps in capacity.rs ----------

#[tokio::test]
async fn get_load_reports_fraction_of_capacity() {
    let t = CapacityTracker::new(4, 1000);
    assert_eq!(t.get_load().await, 0.0);

    t.increment_rooms().await;
    t.increment_matches().await;
    assert!((t.get_load().await - 0.5).abs() < 1e-9);

    t.increment_rooms().await;
    t.increment_matches().await;
    assert_eq!(t.get_load().await, 1.0);
}

#[tokio::test]
async fn get_stats_reflects_room_and_match_counts() {
    let t = CapacityTracker::new(10, 1000);
    t.increment_rooms().await;
    t.increment_rooms().await;
    t.increment_matches().await;
    let s = t.get_stats().await;
    assert_eq!(s.active_rooms, 2);
    assert_eq!(s.active_matches, 1);
    assert_eq!(s.total_active, 3);
    assert_eq!(s.max_capacity, 10);
    assert_eq!(s.load_percentage, 30);
}

#[tokio::test]
async fn decrement_rooms_saturates_at_zero() {
    let t = CapacityTracker::new(4, 1000);
    t.decrement_rooms().await;
    t.decrement_rooms().await;
    let s = t.get_stats().await;
    assert_eq!(s.active_rooms, 0);
}

#[tokio::test]
async fn decrement_matches_saturates_at_zero() {
    let t = CapacityTracker::new(4, 1000);
    t.decrement_matches().await;
    let s = t.get_stats().await;
    assert_eq!(s.active_matches, 0);
}

#[tokio::test]
async fn increment_then_decrement_room_round_trip() {
    let t = CapacityTracker::new(4, 1000);
    t.increment_rooms().await;
    t.increment_rooms().await;
    t.decrement_rooms().await;
    let s = t.get_stats().await;
    assert_eq!(s.active_rooms, 1);
}

// ---------- judge game registry ----------

#[test]
fn games_registry_has_three_known_ids() {
    let ids: Vec<&str> = GAMES.iter().map(|g| g.id).collect();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"rock-paper-scissors"));
    assert!(ids.contains(&"tic-tac-toe"));
    assert!(ids.contains(&"prisoners-dilemma"));
}

#[test]
fn find_game_by_id_known_returns_metadata() {
    let g = find_game_by_id("rock-paper-scissors").unwrap();
    assert_eq!(g.id, "rock-paper-scissors");
    assert!(g.bot_turn_timeout_ms > 0);
    assert!(g.memory_limit_mb > 0);
}

#[test]
fn find_game_by_id_unknown_is_none() {
    assert!(find_game_by_id("not-a-game").is_none());
}

// ---------- RoomLogic statics per game ----------

#[test]
fn room_logic_static_facts_match_docs() {
    assert_eq!(Rps::game_id(), "rock-paper-scissors");
    assert_eq!(Rps::max_players(), 2);
    assert_eq!(Ttt::game_id(), "tic-tac-toe");
    assert_eq!(Ttt::max_players(), 2);
    assert_eq!(Pd::game_id(), "prisoners-dilemma");
    assert_eq!(Pd::max_players(), 2);
}

#[test]
fn snapshot_reflects_default_lobby_phase() {
    let st = <Rps as RoomLogic>::State::default();
    let snap = Rps::snapshot(&st);
    assert_eq!(snap.phase, "lobby");
    assert!(snap.players.is_empty());
    assert!(snap.host.is_none());
}

// ---------- validate_jwt edges ----------

fn sign(claims: &Claims, secret: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[test]
fn validate_jwt_accepts_valid_token() {
    let secret = "judge-coverage-secret";
    let claims = Claims {
        sub: "user:alice".into(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    let tok = sign(&claims, secret);
    let parsed = validate_jwt(&tok, secret).unwrap();
    assert_eq!(parsed.sub, "user:alice");
}

#[test]
fn validate_jwt_rejects_expired_token() {
    let secret = "judge-coverage-secret";
    let claims = Claims {
        sub: "user:alice".into(),
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
    };
    let tok = sign(&claims, secret);
    assert!(validate_jwt(&tok, secret).is_err());
}

#[test]
fn validate_jwt_rejects_wrong_secret() {
    let claims = Claims {
        sub: "user:alice".into(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    let tok = sign(&claims, "secret-A");
    assert!(validate_jwt(&tok, "secret-B").is_err());
}

#[test]
fn validate_jwt_rejects_garbage() {
    assert!(validate_jwt("not.a.jwt", "any-secret").is_err());
    assert!(validate_jwt("", "any-secret").is_err());
}

// ---------- Storage factories ----------

#[tokio::test]
async fn memory_storage_factory_yields_independent_instances() {
    let a = Storage::memory();
    let b = Storage::memory();

    a.lease.acquire("r", "owner-A", std::time::Duration::from_secs(60))
        .await
        .unwrap();
    // Storage::memory() must not share state — the second factory call
    // is a fresh in-memory backend.
    assert!(b.lease.acquire("r", "owner-B", std::time::Duration::from_secs(60))
        .await
        .unwrap());
}

#[tokio::test]
async fn surreal_storage_factory_round_trips_event() {
    let db = db::setup_test_db().await;
    let storage = Storage::surreal(db);

    let room = unique("room");
    let owner = unique("owner");

    storage
        .lease
        .acquire(&room, &owner, std::time::Duration::from_secs(60))
        .await
        .unwrap();
    let e = storage
        .log
        .append(&room, &owner, "PLAYER_JOINED", "alice")
        .await
        .unwrap();
    assert_eq!(e.seq, 1);
    assert_eq!(e.kind, "PLAYER_JOINED");
    storage.lease.release(&room, &owner).await.unwrap();
}

// ---------- Claims serde ----------

#[test]
fn claims_serde_round_trip() {
    let c = Claims {
        sub: "user:bob".into(),
        exp: 1700000000,
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: Claims = serde_json::from_str(&json).unwrap();
    assert_eq!(back.sub, "user:bob");
    assert_eq!(back.exp, 1700000000);
}

// ---------- DB connect via env ----------

#[tokio::test]
async fn db_connect_uses_env_config() {
    let cfg = Config::from_env();
    let db = judge::db::connect(
        &cfg.database_url,
        &cfg.database_ns,
        &cfg.database_db,
        &cfg.database_user,
        &cfg.database_pass,
    )
    .await
    .unwrap();
    let mut resp = db.query("RETURN 1 + 1").await.unwrap();
    let v: Option<i64> = resp.take(0).unwrap();
    assert_eq!(v, Some(2));
}
