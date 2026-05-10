// Plumbing tests covering integration seams between subsystems:
//   * matchmaking dequeue actually pulls a player from the queue.
//   * healer::tick auto-flips Scheduled -> Registration when start_time
//     is reached, and finalizes Running tournaments whose matches are
//     all terminal.
//   * Multi-bot upload + select: tournament start picks the user's
//     selected bot, not the first one uploaded.

mod db;

use api::{
    config::Config,
    models::{
        matches::Match, MatchGenerationType, ProgrammingLanguage, TournamentKind,
        TournamentStatus,
    },
    services::{auth, healer, matchmaking, submission, tournament},
};
use surrealdb::types::SurrealValue;
use chrono::{Duration as ChDuration, Utc};
use surrealdb::types::RecordId;

const GAME: &str = "rock-paper-scissors";

fn unique(p: &str) -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{p}{n}")
}

async fn user_id(db: &api::db::Database, email: &str) -> RecordId {
    auth::get_user_by_email(db, email)
        .await
        .unwrap()
        .unwrap()
        .id
        .unwrap()
}

async fn accept(db: &api::db::Database, sid: &RecordId) {
    db.query(
        "UPDATE $sid SET status='accepted', compiled_binary_path='/tmp/test-bin'",
    )
    .bind(("sid", sid.clone()))
    .await
    .unwrap();
}

#[tokio::test]
async fn matchmaking_dequeue_removes_user_from_queue() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;
    let alice = user_id(&db, &cfg.alice.email).await;

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("MM "),
        "mm".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Human),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    tournament::join_tournament(&db, tid.clone(), bob.clone()).await.unwrap();
    tournament::join_tournament(&db, tid.clone(), alice.clone()).await.unwrap();
    db.query("UPDATE $tid SET status='running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    matchmaking::enqueue(&db, tid.clone(), bob.clone()).await.unwrap();
    matchmaking::enqueue(&db, tid.clone(), alice.clone()).await.unwrap();

    // Bob bails before pairing happens.
    matchmaking::dequeue(&db, tid.clone(), bob).await.unwrap();

    let paired = matchmaking::tick(&db, tid.clone()).await.unwrap();
    assert_eq!(paired, 0, "only one user left in queue, no pairing");

    let mut resp = db
        .query("SELECT * FROM matchmaking_ticket WHERE tournament_id=$tid")
        .bind(("tid", tid))
        .await
        .unwrap();
    let tickets: Vec<matchmaking::MatchmakingTicket> = resp.take(0).unwrap();
    assert_eq!(tickets.len(), 1, "alice's ticket remains for next time");
    assert_eq!(tickets[0].user_id, alice);
}

#[tokio::test]
async fn healer_promotes_scheduled_tournament_when_start_time_passes() {
    let db = db::setup_test_db().await;
    let past = Utc::now() - ChDuration::minutes(5);

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("Sched "),
        "scheduled".into(),
        2,
        4,
        Some(past),
        None,
        None,
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();

    // Force Scheduled state (create_tournament defaults to Registration).
    db.query("UPDATE $tid SET status='scheduled'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    healer::tick(&db).await;

    let after = tournament::get_tournament(&db, tid).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Registration);
}

#[tokio::test]
async fn healer_does_not_promote_scheduled_with_future_start_time() {
    let db = db::setup_test_db().await;
    let future = Utc::now() + ChDuration::hours(1);

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("FutureSched "),
        "future".into(),
        2,
        4,
        Some(future),
        None,
        None,
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    db.query("UPDATE $tid SET status='scheduled'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    healer::tick(&db).await;

    let after = tournament::get_tournament(&db, tid).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Scheduled);
}

#[tokio::test]
async fn healer_finalizes_running_tournament_when_all_matches_terminal() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;
    let alice = user_id(&db, &cfg.alice.email).await;

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("HealFin "),
        "heal".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for uid in [bob.clone(), alice.clone()] {
        tournament::join_tournament(&db, tid.clone(), uid.clone()).await.unwrap();
        let s = submission::create_submission(
            &db,
            uid,
            tid.clone(),
            GAME.into(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept(&db, s.id.as_ref().unwrap()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    let mut resp = db
        .query("SELECT * FROM match WHERE tournament_id=$tid")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    let ms: Vec<Match> = resp.take(0).unwrap();
    let mid = ms.into_iter().next().unwrap().id.unwrap();
    db.query(
        "UPDATE $mid SET status='completed',
                          participants[0].score=2.0,
                          participants[1].score=1.0,
                          completed_at=time::now()",
    )
    .bind(("mid", mid))
    .await
    .unwrap();

    healer::tick(&db).await;

    let after = tournament::get_tournament(&db, tid).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Completed);
}

#[tokio::test]
async fn healer_tick_advances_single_elim_bracket_without_explicit_call() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;
    let alice = user_id(&db, &cfg.alice.email).await;

    // Two extra synthetic players for a 4-player single-elim.
    async fn ensure_user(db: &api::db::Database, suffix: u32) -> surrealdb::types::RecordId {
        let username = format!("healer_user_{suffix}");
        let email = format!("{username}@test.local");
        if let Some(u) = auth::get_user_by_email(db, &email).await.unwrap() {
            return u.id.unwrap();
        }
        let mut resp = db
            .query(
                "CREATE user SET email=$e, username=$u, role='player',
                              location='US', is_banned=false,
                              created_at=time::now(), updated_at=time::now()
                 RETURN AFTER",
            )
            .bind(("e", email))
            .bind(("u", username))
            .await
            .unwrap();
        let rows: Vec<api::models::User> = resp.take(0).unwrap();
        rows.into_iter().next().unwrap().id.unwrap()
    }

    let players = vec![
        bob.clone(),
        alice.clone(),
        ensure_user(&db, 91).await,
        ensure_user(&db, 92).await,
    ];

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("HealAdv "),
        "healer drives bracket".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::SingleElimination),
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for p in &players {
        tournament::join_tournament(&db, tid.clone(), p.clone()).await.unwrap();
        let s = submission::create_submission(
            &db,
            p.clone(),
            tid.clone(),
            GAME.into(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept(&db, s.id.as_ref().unwrap()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    // Settle round 0: both index-0 wins.
    #[derive(serde::Deserialize, SurrealValue)]
    struct Row {
        id: surrealdb::types::RecordId,
    }
    let mut resp = db
        .query("SELECT id FROM match WHERE tournament_id=$tid AND round=0")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    let r0: Vec<Row> = resp.take(0).unwrap();
    for row in r0 {
        db.query(
            "UPDATE $mid SET status='completed',
                              participants[0].score=1.0,
                              participants[1].score=0.0,
                              completed_at=time::now()",
        )
        .bind(("mid", row.id))
        .await
        .unwrap();
    }

    // Healer tick alone must materialise round 1.
    healer::tick(&db).await;

    let mut resp = db
        .query("SELECT id FROM match WHERE tournament_id=$tid AND round=1")
        .bind(("tid", tid))
        .await
        .unwrap();
    let r1: Vec<Row> = resp.take(0).unwrap();
    assert_eq!(
        r1.len(),
        1,
        "healer must drive bracket advance: round-1 match should exist"
    );
}

#[tokio::test]
async fn concurrent_matchmaking_ticks_do_not_double_pair() {
    // Two ticks racing on the same queue must collectively create at
    // most floor(n/2) rooms, and no user must appear in more than one
    // room. The atomic ticket-claim in matchmaking::tick is what makes
    // this safe.
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;
    let alice = user_id(&db, &cfg.alice.email).await;

    async fn ensure_user(db: &api::db::Database, suffix: &str) -> RecordId {
        let username = format!("mm_user_{suffix}");
        let email = format!("{username}@test.local");
        if let Some(u) = auth::get_user_by_email(db, &email).await.unwrap() {
            return u.id.unwrap();
        }
        let mut resp = db
            .query(
                "CREATE user SET email=$e, username=$u, role='player',
                              location='US', is_banned=false,
                              created_at=time::now(), updated_at=time::now()
                 RETURN AFTER",
            )
            .bind(("e", email))
            .bind(("u", username))
            .await
            .unwrap();
        let rows: Vec<api::models::User> = resp.take(0).unwrap();
        rows.into_iter().next().unwrap().id.unwrap()
    }

    let nonce = unique("");
    let players = vec![
        bob.clone(),
        alice.clone(),
        ensure_user(&db, &format!("c_{nonce}")).await,
        ensure_user(&db, &format!("d_{nonce}")).await,
    ];

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("MMRace "),
        "race".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Human),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for p in &players {
        tournament::join_tournament(&db, tid.clone(), p.clone()).await.unwrap();
    }
    db.query("UPDATE $tid SET status='running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    for p in &players {
        matchmaking::enqueue(&db, tid.clone(), p.clone()).await.unwrap();
    }

    // Spawn two concurrent ticks.
    let (db1, tid1) = (db.clone(), tid.clone());
    let (db2, tid2) = (db.clone(), tid.clone());
    let h1 = tokio::spawn(async move { matchmaking::tick(&db1, tid1).await.unwrap() });
    let h2 = tokio::spawn(async move { matchmaking::tick(&db2, tid2).await.unwrap() });
    let paired_a = h1.await.unwrap();
    let paired_b = h2.await.unwrap();
    let total = paired_a + paired_b;
    assert!(
        total <= 2,
        "two ticks across 4 queued players must produce <= 2 rooms, got {total} (a={paired_a}, b={paired_b})"
    );

    // No user must appear in more than one room.
    #[derive(serde::Deserialize, SurrealValue)]
    struct R {
        players: Vec<RecordId>,
    }
    let mut resp = db
        .query("SELECT players FROM room WHERE tournament_id=$tid AND is_ranked=true")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    let rooms: Vec<R> = resp.take(0).unwrap();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &rooms {
        for u in &r.players {
            assert!(
                seen.insert(format!("{u:?}")),
                "user {u:?} placed in two ranked rooms — race not contained"
            );
        }
    }
    assert_eq!(rooms.len() as u32, total, "room count must match paired count");
}

#[tokio::test]
async fn submission_rate_limit_rejects_after_threshold() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("RL "),
        "rl".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    tournament::join_tournament(&db, tid.clone(), bob.clone()).await.unwrap();

    // Spam up to the limit; all should succeed.
    for i in 0..api::services::submission::RATE_LIMIT {
        submission::create_submission(
            &db,
            bob.clone(),
            tid.clone(),
            GAME.into(),
            ProgrammingLanguage::Rust,
            format!("fn main(){{ /* {i} */ }}"),
        )
        .await
        .unwrap();
    }
    // One more should fail.
    let r = submission::create_submission(
        &db,
        bob.clone(),
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        "fn main(){ /* over */ }".into(),
    )
    .await;
    assert!(r.is_err(), "rate limit must reject submission #{}", api::services::submission::RATE_LIMIT + 1);
    let msg = format!("{}", r.unwrap_err());
    assert!(msg.to_lowercase().contains("rate limit"), "expected rate limit message, got: {msg}");
}

#[tokio::test]
async fn submission_size_cap_rejects_oversized_code() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("SizeCap "),
        "size".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    tournament::join_tournament(&db, tid.clone(), bob.clone()).await.unwrap();

    // Just under the cap: must succeed.
    let mut ok = String::with_capacity(api::services::submission::MAX_CODE_BYTES);
    while ok.len() < api::services::submission::MAX_CODE_BYTES - 32 {
        ok.push('x');
    }
    submission::create_submission(
        &db,
        bob.clone(),
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        ok,
    )
    .await
    .expect("under-cap submission should succeed");

    // Over the cap by 1 byte: must reject.
    let big = "y".repeat(api::services::submission::MAX_CODE_BYTES + 1);
    let r = submission::create_submission(
        &db,
        bob.clone(),
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        big,
    )
    .await;
    assert!(r.is_err(), "oversized submission must be rejected");
    let msg = format!("{}", r.unwrap_err()).to_lowercase();
    assert!(
        msg.contains("too large") || msg.contains("size"),
        "expected size-limit error, got: {msg}"
    );
}

#[tokio::test]
async fn multi_bot_upload_then_select_makes_v2_active_for_start() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = user_id(&db, &cfg.bob.email).await;
    let alice = user_id(&db, &cfg.alice.email).await;

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("Multi "),
        "multi".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
        Some(TournamentKind::Bot),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    tournament::join_tournament(&db, tid.clone(), bob.clone()).await.unwrap();
    tournament::join_tournament(&db, tid.clone(), alice.clone()).await.unwrap();

    // Bob uploads two bots, picks v2.
    let v1 = submission::create_submission(
        &db,
        bob.clone(),
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        "fn main(){/*v1*/}".into(),
    )
    .await
    .unwrap();
    accept(&db, v1.id.as_ref().unwrap()).await;
    let v2 = submission::create_submission(
        &db,
        bob.clone(),
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        "fn main(){/*v2*/}".into(),
    )
    .await
    .unwrap();
    accept(&db, v2.id.as_ref().unwrap()).await;
    submission::select_active_submission(&db, bob.clone(), v2.id.clone().unwrap())
        .await
        .unwrap();

    // Alice uploads one bot.
    let alice_sub = submission::create_submission(
        &db,
        alice.clone(),
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        "fn main(){}".into(),
    )
    .await
    .unwrap();
    accept(&db, alice_sub.id.as_ref().unwrap()).await;

    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    let mut resp = db
        .query("SELECT * FROM match WHERE tournament_id=$tid")
        .bind(("tid", tid))
        .await
        .unwrap();
    let ms: Vec<Match> = resp.take(0).unwrap();
    assert_eq!(ms.len(), 1);
    let bob_part = ms[0]
        .participants
        .iter()
        .find(|p| p.user_id == bob)
        .unwrap();
    assert_eq!(
        bob_part.submission_id.as_ref().unwrap(),
        v2.id.as_ref().unwrap(),
        "match must reference bob's selected v2 bot, not v1"
    );
}
