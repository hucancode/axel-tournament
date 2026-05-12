// Bot tournament end-to-end coverage.
//
// Each phase of the spec is exercised against the real DB:
//   * Join + upload bot (multi-bot per user, only first auto-selected).
//   * Select active bot.
//   * Admin start + match generation.
//   * Match outcomes (completed, failed) flow through finalization.
//   * Score aggregation, ranks, tournament transitions to completed.
//   * Per-bot stats (wins/losses/draws/total_score).

mod db;

use api::{
    config::Config,
    models::{
        matches::{Match, MatchStatus},
        ProgrammingLanguage, TournamentKind, TournamentStatus,
    },
    services::{finalization, matches, stats, submission, tournament},
};
use surrealdb::types::{Datetime, RecordId};

const GAME: &str = "rock-paper-scissors";

fn unique(prefix: &str) -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}{n}")
}

async fn bob(db: &api::db::Database) -> api::models::User {
    let cfg = Config::from_env();
    <api::db::Database as axel_core::repo::user::UserRepo>::find_by_email(db, &cfg.bob.email)
        .await
        .unwrap()
        .expect("bob exists")
}

async fn alice(db: &api::db::Database) -> api::models::User {
    let cfg = Config::from_env();
    <api::db::Database as axel_core::repo::user::UserRepo>::find_by_email(db, &cfg.alice.email)
        .await
        .unwrap()
        .expect("alice exists")
}

/// Force a submission to `accepted`, simulating the judge compiler
/// having finished. Without this, `start_tournament` filters the
/// participant out and tests can't generate matches.
async fn accept_submission(db: &api::db::Database, sid: &RecordId) {
    db.query(
        "UPDATE $sid SET status = 'accepted',
                          compiled_binary_path = '/tmp/test-bin',
                          error_message = NONE",
    )
    .bind(("sid", sid.clone()))
    .await
    .unwrap();
}

async fn fresh_tournament(db: &api::db::Database) -> RecordId {
    let t = tournament::create_tournament(
        db,
        GAME.to_string(),
        unique("Bot Tour "),
        "test".into(),
        2,
        16,
        None,
        None,
        None,
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    t.id.unwrap()
}

#[tokio::test]
async fn join_then_upload_creates_submission_and_auto_selects() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db).await;
    let bob = bob(&db).await;
    let bob_id = bob.id.unwrap();

    tournament::join_tournament(&db, tid.clone(), bob_id.clone())
        .await
        .unwrap();

    let s = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){}".into(),
    )
    .await
    .unwrap();

    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let me = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    assert_eq!(me.submission_id.as_ref().unwrap(), s.id.as_ref().unwrap());
}

#[tokio::test]
async fn second_upload_does_not_overwrite_active_selection() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db).await;
    let bob = bob(&db).await;
    let bob_id = bob.id.unwrap();

    tournament::join_tournament(&db, tid.clone(), bob_id.clone())
        .await
        .unwrap();

    let first = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){/*v1*/}".into(),
    )
    .await
    .unwrap();
    let _second = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){/*v2*/}".into(),
    )
    .await
    .unwrap();

    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let me = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    assert_eq!(
        me.submission_id.as_ref().unwrap(),
        first.id.as_ref().unwrap(),
        "v1 must remain selected after v2 upload"
    );
}

#[tokio::test]
async fn select_active_submission_swaps_selection() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db).await;
    let bob = bob(&db).await;
    let bob_id = bob.id.unwrap();

    tournament::join_tournament(&db, tid.clone(), bob_id.clone())
        .await
        .unwrap();

    let _v1 = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){/*v1*/}".into(),
    )
    .await
    .unwrap();
    let v2 = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){/*v2*/}".into(),
    )
    .await
    .unwrap();

    submission::select_active_submission(&db, bob_id.clone(), v2.id.clone().unwrap())
        .await
        .unwrap();

    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let me = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    assert_eq!(
        me.submission_id.as_ref().unwrap(),
        v2.id.as_ref().unwrap(),
        "select endpoint must swap selection to v2"
    );
}

#[tokio::test]
async fn select_other_users_submission_is_forbidden() {
    let db = db::setup_test_db().await;
    let tid = fresh_tournament(&db).await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;
    let bob_id = bob.id.unwrap();
    let alice_id = alice.id.unwrap();

    tournament::join_tournament(&db, tid.clone(), bob_id.clone())
        .await
        .unwrap();

    let bob_sub = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){}".into(),
    )
    .await
    .unwrap();

    let r = submission::select_active_submission(&db, alice_id, bob_sub.id.unwrap()).await;
    assert!(r.is_err(), "alice cannot pick bob's submission");
}

#[tokio::test]
async fn admin_start_generates_matches_for_round_robin() {
    use api::models::MatchGenerationType;
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("RR "),
        "rr".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();

    for u in [bob(&db).await, alice(&db).await] {
        let uid = u.id.unwrap();
        tournament::join_tournament(&db, tid.clone(), uid.clone())
            .await
            .unwrap();
        let s = submission::create_submission(
            &db,
            uid,
            tid.clone(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept_submission(&db, s.id.as_ref().unwrap()).await;
    }

    let started = tournament::start_tournament(&db, tid.clone()).await.unwrap();
    assert_eq!(started.status, TournamentStatus::Running);
    let ms = matches::list_matches(&db, Some(tid), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(ms.len(), 1, "2 players round-robin = 1 unique match");
}

/// Forge a terminal match row in DB so finalization can be exercised
/// without spinning the judge. Mirrors what judge writes on completion.
#[allow(dead_code)]
async fn write_completed_match(
    db: &api::db::Database,
    tid: RecordId,
    p1: RecordId,
    p2: RecordId,
    s1: f64,
    s2: f64,
) -> RecordId {
    use api::models::matches::MatchParticipant;
    let m = Match {
        id: None,
        tournament_id: Some(tid),
        game_id: GAME.into(),
        status: MatchStatus::Completed,
        participants: vec![
            MatchParticipant {
                user_id: p1,
                submission_id: None,
                score: Some(s1),
            },
            MatchParticipant {
                user_id: p2,
                submission_id: None,
                score: Some(s2),
            },
        ],
        metadata: None,
        room_id: None,
        game_event_source: None,
        judge_server_name: None,
        error_message: None,
        faulted_user_ids: Vec::new(),
        round: None,
        bracket: None,
        bracket_position: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: Some(Datetime::default()),
        elo_applied: false,
    };
    let created: Option<Match> = db.create("match").content(m).await.unwrap();
    created.unwrap().id.unwrap()
}

async fn write_failed_match(
    db: &api::db::Database,
    tid: RecordId,
    p1: RecordId,
    p2: RecordId,
    reason: &str,
) -> RecordId {
    use api::models::matches::MatchParticipant;
    let m = Match {
        id: None,
        tournament_id: Some(tid),
        game_id: GAME.into(),
        status: MatchStatus::Failed,
        participants: vec![
            MatchParticipant {
                user_id: p1,
                submission_id: None,
                score: None,
            },
            MatchParticipant {
                user_id: p2,
                submission_id: None,
                score: None,
            },
        ],
        metadata: None,
        room_id: None,
        game_event_source: None,
        judge_server_name: None,
        error_message: Some(reason.to_string()),
        faulted_user_ids: Vec::new(),
        round: None,
        bracket: None,
        bracket_position: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: Some(Datetime::default()),
        elo_applied: false,
    };
    let created: Option<Match> = db.create("match").content(m).await.unwrap();
    created.unwrap().id.unwrap()
}

#[tokio::test]
async fn finalize_aggregates_scores_and_marks_completed() {
    let db = db::setup_test_db().await;

    // Create + start with a single match so tournament is in Running.
    use api::models::MatchGenerationType;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Finalize "),
        "fin".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    let bob_id = bob(&db).await.id.unwrap();
    let alice_id = alice(&db).await.id.unwrap();
    for uid in [bob_id.clone(), alice_id.clone()] {
        tournament::join_tournament(&db, tid.clone(), uid.clone())
            .await
            .unwrap();
        let s = submission::create_submission(
            &db,
            uid,
            tid.clone(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept_submission(&db, s.id.as_ref().unwrap()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    // Mark generated match as completed: bob 3 vs alice 1.
    let mut resp = db
        .query("SELECT * FROM match WHERE tournament_id = $tid")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    let ms: Vec<Match> = resp.take(0).unwrap();
    let m = ms.into_iter().next().unwrap();
    let mid = m.id.unwrap();
    db.query(
        "UPDATE $mid SET status = 'completed',
                          participants[0].score = 3.0,
                          participants[1].score = 1.0,
                          completed_at = time::now()",
    )
    .bind(("mid", mid))
    .await
    .unwrap();

    // Finalize.
    let updated = finalization::finalize_if_done(&db, tid.clone()).await.unwrap();
    assert_eq!(updated.status, TournamentStatus::Completed);

    // Score + ranks recorded on participants.
    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let me_bob = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    let me_alice = parts.iter().find(|p| p.user_id == alice_id).unwrap();
    // bob's match score is whatever index he sits at (0 or 1). Just
    // verify w/l/d add up and one player is ranked above the other.
    let (winner, loser) = if me_bob.score > me_alice.score {
        (me_bob, me_alice)
    } else {
        (me_alice, me_bob)
    };
    assert_eq!(winner.wins, 1);
    assert_eq!(loser.losses, 1);
    assert_eq!(winner.rank, Some(1));
    assert_eq!(loser.rank, Some(2));
}

#[tokio::test]
async fn finalize_does_not_complete_with_pending_matches() {
    use api::models::MatchGenerationType;
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Half "),
        "half".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::AllVsAll),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for u in [bob(&db).await, alice(&db).await] {
        let uid = u.id.unwrap();
        tournament::join_tournament(&db, tid.clone(), uid.clone()).await.unwrap();
        let s = submission::create_submission(
            &db,
            uid,
            tid.clone(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept_submission(&db, s.id.as_ref().unwrap()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();
    // 2x2 = 4 matches generated; finish only 1 of them.
    let mut resp = db
        .query("SELECT id FROM match WHERE tournament_id = $tid LIMIT 1")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    use surrealdb::types::SurrealValue;
    #[derive(serde::Deserialize, SurrealValue)]
    struct Row {
        id: RecordId,
    }
    let rows: Vec<Row> = resp.take(0).unwrap();
    let one = rows.into_iter().next().unwrap().id;
    db.query("UPDATE $mid SET status = 'completed', participants[0].score = 1.0, participants[1].score = 0.0")
        .bind(("mid", one))
        .await
        .unwrap();

    let updated = finalization::finalize_if_done(&db, tid).await.unwrap();
    assert_eq!(
        updated.status,
        TournamentStatus::Running,
        "still has pending matches"
    );
}

#[tokio::test]
async fn finalize_treats_failed_match_as_loss_for_each_side() {
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Failed "),
        "f".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    let bob_id = bob(&db).await.id.unwrap();
    let alice_id = alice(&db).await.id.unwrap();
    for uid in [bob_id.clone(), alice_id.clone()] {
        tournament::join_tournament(&db, tid.clone(), uid.clone()).await.unwrap();
        submission::create_submission(
            &db,
            uid,
            tid.clone(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
    }
    // Manually force tournament -> running so finalization runs.
    db.query("UPDATE $tid SET status = 'running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    let _ = write_failed_match(&db, tid.clone(), bob_id.clone(), alice_id.clone(), "compile_error").await;

    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();

    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let bob_p = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    let alice_p = parts.iter().find(|p| p.user_id == alice_id).unwrap();
    assert_eq!(bob_p.losses, 1);
    assert_eq!(alice_p.losses, 1);
    assert_eq!(bob_p.wins, 0);
    assert_eq!(alice_p.wins, 0);
}

#[tokio::test]
async fn runtime_error_only_punishes_the_faulted_bot() {
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Crash "),
        "rt".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    let bob_id = bob(&db).await.id.unwrap();
    let alice_id = alice(&db).await.id.unwrap();
    for uid in [bob_id.clone(), alice_id.clone()] {
        tournament::join_tournament(&db, tid.clone(), uid.clone()).await.unwrap();
        let s = submission::create_submission(
            &db,
            uid,
            tid.clone(),
            ProgrammingLanguage::Rust,
            "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept_submission(&db, s.id.as_ref().unwrap()).await;
    }
    db.query("UPDATE $tid SET status = 'running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    // Forge a failed match where only bob crashed.
    use api::models::matches::{Match, MatchParticipant, MatchStatus};
    let m = Match {
        id: None,
        tournament_id: Some(tid.clone()),
        game_id: GAME.into(),
        status: MatchStatus::Failed,
        participants: vec![
            MatchParticipant { user_id: bob_id.clone(), submission_id: None, score: Some(0.0) },
            MatchParticipant { user_id: alice_id.clone(), submission_id: None, score: Some(0.0) },
        ],
        metadata: None,
        room_id: None,
        game_event_source: None,
        judge_server_name: None,
        error_message: Some("runtime_error".into()),
        faulted_user_ids: vec![bob_id.clone()],
        round: None,
        bracket: None,
        bracket_position: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: Some(Datetime::default()),
        elo_applied: false,
    };
    let _: Option<Match> = db.create("match").content(m).await.unwrap();

    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();
    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let bob_p = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    let alice_p = parts.iter().find(|p| p.user_id == alice_id).unwrap();
    assert_eq!(bob_p.losses, 1);
    assert_eq!(bob_p.wins, 0);
    assert_eq!(alice_p.wins, 1);
    assert_eq!(alice_p.losses, 0);
}

#[tokio::test]
async fn illegal_move_only_punishes_the_offender() {
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Cheat "),
        "il".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    let bob_id = bob(&db).await.id.unwrap();
    let alice_id = alice(&db).await.id.unwrap();
    for uid in [bob_id.clone(), alice_id.clone()] {
        tournament::join_tournament(&db, tid.clone(), uid.clone()).await.unwrap();
        let s = submission::create_submission(
            &db, uid, tid.clone(), ProgrammingLanguage::Rust, "fn main(){}".into(),
        )
        .await
        .unwrap();
        accept_submission(&db, s.id.as_ref().unwrap()).await;
    }
    db.query("UPDATE $tid SET status = 'running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    use api::models::matches::{Match, MatchParticipant, MatchStatus};
    let m = Match {
        id: None,
        tournament_id: Some(tid.clone()),
        game_id: GAME.into(),
        status: MatchStatus::Failed,
        participants: vec![
            MatchParticipant { user_id: bob_id.clone(), submission_id: None, score: Some(0.0) },
            MatchParticipant { user_id: alice_id.clone(), submission_id: None, score: Some(0.0) },
        ],
        metadata: None,
        room_id: None,
        game_event_source: None,
        judge_server_name: None,
        error_message: Some("illegal_move".into()),
        faulted_user_ids: vec![alice_id.clone()],
        round: None,
        bracket: None,
        bracket_position: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: Some(Datetime::default()),
        elo_applied: false,
    };
    let _: Option<Match> = db.create("match").content(m).await.unwrap();

    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();
    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let bob_p = parts.iter().find(|p| p.user_id == bob_id).unwrap();
    let alice_p = parts.iter().find(|p| p.user_id == alice_id).unwrap();
    assert_eq!(bob_p.wins, 1);
    assert_eq!(alice_p.losses, 1);
}

#[tokio::test]
async fn start_tournament_excludes_uncompiled_bots() {
    use api::models::MatchGenerationType;
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Pending "),
        "p".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::RoundRobin),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    let bob_id = bob(&db).await.id.unwrap();
    let alice_id = alice(&db).await.id.unwrap();
    tournament::join_tournament(&db, tid.clone(), bob_id.clone()).await.unwrap();
    tournament::join_tournament(&db, tid.clone(), alice_id.clone()).await.unwrap();

    // Bob: accepted. Alice: still pending.
    let bob_sub = submission::create_submission(
        &db, bob_id.clone(), tid.clone(), ProgrammingLanguage::Rust, "fn main(){}".into(),
    ).await.unwrap();
    accept_submission(&db, bob_sub.id.as_ref().unwrap()).await;
    let _alice_sub = submission::create_submission(
        &db, alice_id.clone(), tid.clone(), ProgrammingLanguage::Rust, "fn main(){}".into(),
    ).await.unwrap();

    // Only one accepted: below min_players (2). Should error cleanly.
    let r = tournament::start_tournament(&db, tid).await;
    assert!(r.is_err(), "must reject when uncompiled bots leave us below min_players");
}

#[tokio::test]
async fn submission_stats_reports_per_bot_record() {
    let db = db::setup_test_db().await;
    let t = tournament::create_tournament(
        &db,
        GAME.to_string(),
        unique("Stats "),
        "s".into(),
        2,
        4,
        None,
        None,
        None,
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    let bob_id = bob(&db).await.id.unwrap();
    let alice_id = alice(&db).await.id.unwrap();
    for uid in [bob_id.clone(), alice_id.clone()] {
        tournament::join_tournament(&db, tid.clone(), uid.clone()).await.unwrap();
    }

    let bob_sub = submission::create_submission(
        &db,
        bob_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){}".into(),
    )
    .await
    .unwrap();
    let _alice_sub = submission::create_submission(
        &db,
        alice_id.clone(),
        tid.clone(),
        ProgrammingLanguage::Rust,
        "fn main(){}".into(),
    )
    .await
    .unwrap();

    // Forge two matches for bob's submission: one win, one draw.
    use api::models::matches::MatchParticipant;
    let make = |s1: f64, s2: f64| Match {
        id: None,
        tournament_id: Some(tid.clone()),
        game_id: GAME.into(),
        status: MatchStatus::Completed,
        participants: vec![
            MatchParticipant {
                user_id: bob_id.clone(),
                submission_id: bob_sub.id.clone(),
                score: Some(s1),
            },
            MatchParticipant {
                user_id: alice_id.clone(),
                submission_id: None,
                score: Some(s2),
            },
        ],
        metadata: None,
        room_id: None,
        game_event_source: None,
        judge_server_name: None,
        error_message: None,
        faulted_user_ids: Vec::new(),
        round: None,
        bracket: None,
        bracket_position: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: None,
        completed_at: Some(Datetime::default()),
        elo_applied: false,
    };
    let _: Option<Match> = db.create("match").content(make(2.0, 0.0)).await.unwrap();
    let _: Option<Match> = db.create("match").content(make(1.0, 1.0)).await.unwrap();

    let stats = stats::submission_stats(&db, bob_sub.id.unwrap())
        .await
        .unwrap();
    assert_eq!(stats.matches_played, 2);
    assert_eq!(stats.wins, 1);
    assert_eq!(stats.draws, 1);
    assert_eq!(stats.losses, 0);
    assert!((stats.total_score - 3.0).abs() < f64::EPSILON);
}
