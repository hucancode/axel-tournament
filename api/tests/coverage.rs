// Service-level integration tests filling coverage gaps.
//
// Existing per-service test files cover the happy paths of each
// module's main functions. This file fills the remaining gaps so every
// public service function the api modules promise has at least one
// passing test, and every documented edge case (forbidden, conflict,
// not-found) is exercised.
//
// Connection settings come from `Config::from_env()` — no hard-coded
// URLs or credentials. Tests assume SurrealDB and Mailpit are up
// (`make test-db-up`, `make test-mail-server-up`). Each test allocates
// unique emails / usernames so concurrent runs cannot collide.

mod db;

use api::{
    config::Config,
    models::{
        matches::MatchStatus, MatchGenerationType, OAuthProvider, ProgrammingLanguage,
        SubmissionStatus, TournamentStatus,
    },
    services::{
        auth as auth_svc, email as email_svc, matches as match_svc, submission as sub_svc,
        tournament as tour_svc, user as user_svc,
    },
};
use surrealdb::types::RecordId;

fn unique(prefix: &str) -> String {
    format!("{prefix}{}", uuid::Uuid::new_v4().simple())
}

async fn fresh_player(db: &api::db::Database) -> api::models::User {
    let email = format!("{}@example.com", unique("u"));
    let username = unique("user");
    user_svc::create_user(
        db,
        user_svc::NewUser {
            email,
            username,
            password_hash: Some("hashed".to_string()),
            location: "US".to_string(),
            oauth_provider: None,
            oauth_id: None,
        },
    )
    .await
    .unwrap()
}

async fn fresh_tournament(db: &api::db::Database, game_id: &str) -> RecordId {
    let t = tour_svc::create_tournament(
        db,
        game_id.to_string(),
        unique("Tour"),
        "coverage test".to_string(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
        None, None,
    )
    .await
    .unwrap();
    t.id.unwrap()
}

// ---------- User::to_info ----------

#[tokio::test]
async fn user_to_info_projects_user_fields() {
    let db = db::setup_test_db().await;
    let user = fresh_player(&db).await;
    let info = user.to_info().unwrap();
    assert_eq!(info.email, user.email);
    assert_eq!(info.username, user.username);
    assert_eq!(info.location, user.location);
    assert!(!info.is_banned);
    assert!(!info.id.is_empty());
}

// ---------- auth_svc lookups ----------

#[tokio::test]
async fn get_user_by_id_returns_user() {
    let db = db::setup_test_db().await;
    let user = fresh_player(&db).await;
    let by_id = user_svc::get_user_by_id(&db, user.id.clone().unwrap()).await.unwrap();
    assert_eq!(by_id.email, user.email);
}

#[tokio::test]
async fn get_user_by_id_not_found() {
    let db = db::setup_test_db().await;
    let bogus = RecordId::parse_simple("user:does_not_exist").unwrap();
    assert!(user_svc::get_user_by_id(&db, bogus).await.is_err());
}

#[tokio::test]
async fn get_user_by_email_none_for_unknown() {
    let db = db::setup_test_db().await;
    let unknown = format!("{}@nowhere.test", unique("nobody"));
    assert!(user_svc::get_user_by_email(&db, &unknown).await.unwrap().is_none());
}

#[tokio::test]
async fn get_user_by_username_round_trip() {
    let db = db::setup_test_db().await;
    let user = fresh_player(&db).await;
    let by_name = user_svc::get_user_by_username(&db, &user.username).await.unwrap();
    assert_eq!(by_name.unwrap().email, user.email);
    let none = user_svc::get_user_by_username(&db, &unique("ghost")).await.unwrap();
    assert!(none.is_none());
}

#[tokio::test]
async fn get_user_by_oauth_round_trip() {
    let db = db::setup_test_db().await;
    let oauth_id = unique("g");
    let user = user_svc::create_user(
        &db,
        user_svc::NewUser {
            email: format!("{}@oauth.test", unique("u")),
            username: unique("oauth"),
            password_hash: None,
            location: "US".to_string(),
            oauth_provider: Some(OAuthProvider::Google),
            oauth_id: Some(oauth_id.clone()),
        },
    )
    .await
    .unwrap();

    let found = user_svc::get_user_by_oauth(&db, "google", &oauth_id).await.unwrap();
    assert_eq!(found.unwrap().id, user.id);

    let missing = user_svc::get_user_by_oauth(&db, "google", &unique("ghost"))
        .await
        .unwrap();
    assert!(missing.is_none());
}

// ---------- user_svc gaps ----------

#[tokio::test]
async fn create_user_sets_defaults() {
    let db = db::setup_test_db().await;
    let user = fresh_player(&db).await;
    assert_eq!(user.role, api::models::UserRole::Player);
    assert!(!user.is_banned);
    assert!(user.password_reset_token.is_none());
}

// ---------- tournament gaps ----------

#[tokio::test]
async fn list_tournaments_filters_by_status() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;

    let all_reg = tour_svc::list_tournaments(&db, Some(TournamentStatus::Registration), Some(200), None)
        .await
        .unwrap();
    assert!(all_reg.iter().any(|(t, _)| t.id.as_ref() == Some(&tid)));

    tour_svc::update_tournament(
        &db,
        tid.clone(),
        None,
        None,
        Some(TournamentStatus::Cancelled),
        None,
        None,
    )
    .await
    .unwrap();

    let reg_after = tour_svc::list_tournaments(&db, Some(TournamentStatus::Registration), Some(200), None)
        .await
        .unwrap();
    assert!(!reg_after.iter().any(|(t, _)| t.id.as_ref() == Some(&tid)));

    let cancelled = tour_svc::list_tournaments(&db, Some(TournamentStatus::Cancelled), Some(200), None)
        .await
        .unwrap();
    assert!(cancelled.iter().any(|(t, _)| t.id.as_ref() == Some(&tid)));
}

#[tokio::test]
async fn list_tournaments_paginates() {
    let db = db::setup_test_db().await;
    fresh_tournament(&db, "tic-tac-toe").await;
    fresh_tournament(&db, "tic-tac-toe").await;
    fresh_tournament(&db, "tic-tac-toe").await;

    let page = tour_svc::list_tournaments(&db, None, Some(2), Some(0)).await.unwrap();
    assert_eq!(page.len(), 2);
}

#[tokio::test]
async fn list_tournaments_returns_participant_counts() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let alice = fresh_player(&db).await;
    let bob = fresh_player(&db).await;
    tour_svc::join_tournament(&db, tid.clone(), alice.id.clone().unwrap())
        .await
        .unwrap();
    tour_svc::join_tournament(&db, tid.clone(), bob.id.clone().unwrap())
        .await
        .unwrap();

    let listed = tour_svc::list_tournaments(
        &db,
        Some(TournamentStatus::Registration),
        Some(200),
        None,
    )
    .await
    .unwrap();
    let row = listed
        .iter()
        .find(|(t, _)| t.id.as_ref() == Some(&tid))
        .expect("tournament present");
    assert_eq!(row.1, 2);
}

#[tokio::test]
async fn create_tournament_rejects_unknown_game() {
    let db = db::setup_test_db().await;
    let r = tour_svc::create_tournament(
        &db,
        "fake-game".into(),
        unique("T"),
        "x".into(),
        2,
        2,
        None,
        None,
        None,
        None, None,
    )
    .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn join_tournament_rejects_when_not_in_registration() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let user = fresh_player(&db).await;

    tour_svc::update_tournament(
        &db,
        tid.clone(),
        None,
        None,
        Some(TournamentStatus::Running),
        None,
        None,
    )
    .await
    .unwrap();

    let r = tour_svc::join_tournament(&db, tid, user.id.unwrap()).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn join_tournament_conflicts_on_double_join() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let user = fresh_player(&db).await;
    tour_svc::join_tournament(&db, tid.clone(), user.id.clone().unwrap())
        .await
        .unwrap();
    let r = tour_svc::join_tournament(&db, tid, user.id.unwrap()).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn get_tournament_participants_orders_by_score_desc() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let alice = fresh_player(&db).await;
    let bob = fresh_player(&db).await;
    tour_svc::join_tournament(&db, tid.clone(), alice.id.clone().unwrap())
        .await
        .unwrap();
    tour_svc::join_tournament(&db, tid.clone(), bob.id.clone().unwrap())
        .await
        .unwrap();

    // Bump bob's score directly.
    db.query("UPDATE tournament_participant SET score = 5.0f WHERE tournament_id = $tid AND user_id = $uid")
        .bind(("tid", tid.clone()))
        .bind(("uid", bob.id.clone().unwrap()))
        .await
        .unwrap();

    let parts = tour_svc::get_tournament_participants(&db, tid).await.unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].user_id, bob.id.unwrap());
    assert_eq!(parts[1].user_id, alice.id.unwrap());
}

#[tokio::test]
async fn leave_tournament_removes_participant() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let user = fresh_player(&db).await;
    tour_svc::join_tournament(&db, tid.clone(), user.id.clone().unwrap())
        .await
        .unwrap();
    tour_svc::leave_tournament(&db, tid.clone(), user.id.clone().unwrap())
        .await
        .unwrap();
    let parts = tour_svc::get_tournament_participants(&db, tid).await.unwrap();
    assert!(parts.is_empty());
}

// ---------- submission gaps ----------

#[tokio::test]
async fn create_submission_requires_tournament_join() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let user = fresh_player(&db).await;

    // No join — must be Forbidden.
    let r = sub_svc::create_submission(
        &db,
        user.id.unwrap(),
        tid,
        "rock-paper-scissors".to_string(),
        ProgrammingLanguage::Rust,
        "fn main() {}".to_string(),
    )
    .await;
    assert!(matches!(r, Err(api::error::ApiError::Forbidden(_))));
}

#[tokio::test]
async fn update_submission_status_round_trip() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let user = fresh_player(&db).await;
    tour_svc::join_tournament(&db, tid.clone(), user.id.clone().unwrap())
        .await
        .unwrap();
    let s = sub_svc::create_submission(
        &db,
        user.id.unwrap(),
        tid,
        "rock-paper-scissors".to_string(),
        ProgrammingLanguage::Rust,
        "fn main() {}".to_string(),
    )
    .await
    .unwrap();

    let sid = s.id.clone().unwrap();
    let updated = sub_svc::update_submission_status(
        &db,
        sid.clone(),
        SubmissionStatus::Failed,
        Some("compile error".to_string()),
    )
    .await
    .unwrap();
    assert_eq!(updated.status, SubmissionStatus::Failed);
    assert_eq!(updated.error_message.as_deref(), Some("compile error"));

    let cleared = sub_svc::update_submission_status(
        &db,
        sid,
        SubmissionStatus::Accepted,
        None,
    )
    .await
    .unwrap();
    assert_eq!(cleared.status, SubmissionStatus::Accepted);
    assert!(cleared.error_message.is_none());
}

#[tokio::test]
async fn update_submission_status_not_found() {
    let db = db::setup_test_db().await;
    let bogus = RecordId::parse_simple("submission:nope").unwrap();
    let r = sub_svc::update_submission_status(&db, bogus, SubmissionStatus::Accepted, None).await;
    assert!(matches!(r, Err(api::error::ApiError::NotFound(_))));
}

// ---------- matches service ----------

async fn seeded_match(db: &api::db::Database, game_id: &str) -> api::models::matches::Match {
    let tid = fresh_tournament(db, game_id).await;
    let user = fresh_player(db).await;
    tour_svc::join_tournament(db, tid.clone(), user.id.clone().unwrap()).await.unwrap();
    let user2 = fresh_player(db).await;
    tour_svc::join_tournament(db, tid.clone(), user2.id.clone().unwrap()).await.unwrap();

    let s1 = sub_svc::create_submission(
        db,
        user.id.unwrap(),
        tid.clone(),
        game_id.to_string(),
        ProgrammingLanguage::Rust,
        "fn main() {}".to_string(),
    )
    .await
    .unwrap();
    let s2 = sub_svc::create_submission(
        db,
        user2.id.unwrap(),
        tid.clone(),
        game_id.to_string(),
        ProgrammingLanguage::Rust,
        "fn main() {}".to_string(),
    )
    .await
    .unwrap();

    match_svc::create_match(
        db,
        tid,
        game_id.to_string(),
        vec![s1.id.unwrap(), s2.id.unwrap()],
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn create_match_links_submissions() {
    let db = db::setup_test_db().await;
    let m = seeded_match(&db, "rock-paper-scissors").await;
    assert_eq!(m.status, MatchStatus::Pending);
    assert_eq!(m.participants.len(), 2);
    assert!(m.participants.iter().all(|p| p.submission_id.is_some()));
}

#[tokio::test]
async fn create_match_rejects_unknown_game() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let r = match_svc::create_match(&db, tid, "fake-game".to_string(), vec![]).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn create_match_rejects_mismatched_game() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db, "rock-paper-scissors").await;
    let user = fresh_player(&db).await;
    tour_svc::join_tournament(&db, tid.clone(), user.id.clone().unwrap()).await.unwrap();
    let s = sub_svc::create_submission(
        &db,
        user.id.unwrap(),
        tid.clone(),
        "rock-paper-scissors".to_string(),
        ProgrammingLanguage::Rust,
        "fn main() {}".to_string(),
    )
    .await
    .unwrap();

    let r = match_svc::create_match(
        &db,
        tid,
        "tic-tac-toe".to_string(), // wrong game
        vec![s.id.unwrap()],
    )
    .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn get_match_round_trip() {
    let db = db::setup_test_db().await;
    let m = seeded_match(&db, "rock-paper-scissors").await;
    let mid = m.id.clone().unwrap();
    let fetched = match_svc::get_match(&db, mid.clone()).await.unwrap();
    assert_eq!(fetched.id.unwrap(), mid);
}

#[tokio::test]
async fn get_match_not_found() {
    let db = db::setup_test_db().await;
    let bogus = RecordId::parse_simple("match:nope").unwrap();
    assert!(match_svc::get_match(&db, bogus).await.is_err());
}

#[tokio::test]
async fn list_matches_filters_by_tournament() {
    let db = db::setup_test_db().await;
    let m1 = seeded_match(&db, "rock-paper-scissors").await;
    let m2 = seeded_match(&db, "rock-paper-scissors").await;

    let only_t1 = match_svc::list_matches(
        &db,
        Some(m1.tournament_id.clone().unwrap()),
        None,
        None,
        Some(50),
        None,
    )
    .await
    .unwrap();
    assert!(only_t1.iter().any(|m| m.id == m1.id));
    assert!(!only_t1.iter().any(|m| m.id == m2.id));
}

#[tokio::test]
async fn list_matches_filters_by_user() {
    let db = db::setup_test_db().await;
    let m = seeded_match(&db, "tic-tac-toe").await;
    let participant_user_id = m.participants[0].user_id.clone();

    let mine = match_svc::list_matches(&db, None, None, Some(participant_user_id), None, None)
        .await
        .unwrap();
    assert!(mine.iter().any(|x| x.id == m.id));

    let stranger = fresh_player(&db).await;
    let theirs = match_svc::list_matches(&db, None, None, Some(stranger.id.unwrap()), None, None)
        .await
        .unwrap();
    assert!(!theirs.iter().any(|x| x.id == m.id));
}

#[tokio::test]
async fn list_matches_caps_at_max_limit() {
    let db = db::setup_test_db().await;
    // 5000 forced down to 200 internally.
    let r = match_svc::list_matches(&db, None, None, None, Some(5000), None).await.unwrap();
    assert!(r.len() <= 200);
}

// ---------- email service ----------

#[tokio::test]
async fn email_service_rejects_when_credentials_missing() {
    let mut email_cfg = Config::from_env().email;
    email_cfg.smtp_use_tls = true; // exercise the TLS path's credential check
    email_cfg.smtp_username = "".to_string();
    email_cfg.smtp_password = "".to_string();
    let r = email_svc::send_password_reset(&email_cfg, "anyone@example.com", "tok").await;
    assert!(matches!(r, Err(api::error::ApiError::Email(_))));
}

#[tokio::test]
async fn email_service_sends_via_local_smtp() {
    let cfg = Config::from_env();
    let recipient = format!("{}@example.com", unique("u"));
    let r = email_svc::send_password_reset(&cfg.email, &recipient, "test-token").await;
    assert!(r.is_ok(), "send_password_reset failed: {:?}", r.err());
}

// ---------- auth reset token hash determinism ----------

#[tokio::test]
async fn hash_reset_token_is_deterministic() {
    let raw = "deadbeef";
    let h1 = auth_svc::hash_reset_token(raw);
    let h2 = auth_svc::hash_reset_token(raw);
    assert_eq!(h1, h2);
    assert_ne!(h1, auth_svc::hash_reset_token("other"));
}
