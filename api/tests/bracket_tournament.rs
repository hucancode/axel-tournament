// Single-elimination bracket E2E. Validates that tournament start
// emits round-0 matches, advance_brackets pairs winners into round 1,
// and finalize closes the tournament when the final completes.

mod db;

use api::{
    config::Config,
    models::{
        matches::{Match, MatchStatus},
        MatchGenerationType, ProgrammingLanguage, TournamentKind, TournamentStatus,
    },
    services::{bracket, finalization, submission, tournament, user},
};
use surrealdb::types::RecordId;

const GAME: &str = "rock-paper-scissors";

fn unique(prefix: &str) -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}{n}")
}

async fn seeded_user(db: &api::db::Database, email: &str) -> RecordId {
    user::get_user_by_email(db, email)
        .await
        .unwrap()
        .unwrap()
        .id
        .unwrap()
}

async fn ensure_user(db: &api::db::Database, suffix: u32) -> RecordId {
    // Reuse alice/bob plus admin and synthetic users. For >3 we
    // forge user rows directly.
    let username = format!("bracket_user_{suffix}");
    let email = format!("{username}@test.local");
    if let Some(u) = user::get_user_by_email(db, &email).await.unwrap() {
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

async fn register_player(
    db: &api::db::Database,
    tid: &RecordId,
    user: RecordId,
) {
    tournament::join_tournament(db, tid.clone(), user.clone()).await.unwrap();
    let s = submission::create_submission(
        db,
        user,
        tid.clone(),
        GAME.into(),
        ProgrammingLanguage::Rust,
        "fn main(){}".into(),
    )
    .await
    .unwrap();
    db.query(
        "UPDATE $sid SET status = 'accepted',
                          compiled_binary_path = '/tmp/test-bin'",
    )
    .bind(("sid", s.id.unwrap()))
    .await
    .unwrap();
}

async fn complete_match_with_winner(
    db: &api::db::Database,
    mid: RecordId,
    winner_idx: usize,
) {
    let s0 = if winner_idx == 0 { 1.0 } else { 0.0 };
    let s1 = if winner_idx == 1 { 1.0 } else { 0.0 };
    db.query(
        "UPDATE $mid SET status = 'completed',
                         participants[0].score = $s0,
                         participants[1].score = $s1,
                         completed_at = time::now()",
    )
    .bind(("mid", mid))
    .bind(("s0", s0))
    .bind(("s1", s1))
    .await
    .unwrap();
}

async fn list_matches(db: &api::db::Database, tid: &RecordId) -> Vec<Match> {
    let mut resp = db
        .query("SELECT * FROM match WHERE tournament_id = $tid ORDER BY round, bracket_position")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();
    resp.take(0).unwrap()
}

#[tokio::test]
async fn single_elim_two_players_finishes_in_one_match() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let bob = seeded_user(&db, &cfg.bob.email).await;
    let alice = seeded_user(&db, &cfg.alice.email).await;

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("SE2 "),
        "single elim 2".into(),
        2,
        2,
        None,
        None,
        Some(MatchGenerationType::SingleElimination),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    register_player(&db, &tid, bob.clone()).await;
    register_player(&db, &tid, alice.clone()).await;
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    let ms = list_matches(&db, &tid).await;
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].round, Some(0));
    assert_eq!(ms[0].bracket.as_deref(), Some("winners"));

    complete_match_with_winner(&db, ms[0].id.clone().unwrap(), 0).await;
    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();

    let after = tournament::get_tournament(&db, tid).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Completed);
}

#[tokio::test]
async fn single_elim_four_players_runs_two_rounds() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let players = vec![
        seeded_user(&db, &cfg.bob.email).await,
        seeded_user(&db, &cfg.alice.email).await,
        ensure_user(&db, 1).await,
        ensure_user(&db, 2).await,
    ];

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("SE4 "),
        "single elim 4".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::SingleElimination),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for p in &players {
        register_player(&db, &tid, p.clone()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    let r0 = list_matches(&db, &tid).await;
    assert_eq!(r0.len(), 2, "round-0 has 2 matches for 4 players");

    // Settle round 0: index-0 wins both.
    for m in &r0 {
        complete_match_with_winner(&db, m.id.clone().unwrap(), 0).await;
    }

    // Advance.
    let made = bracket::advance_brackets(&db, tid.clone()).await.unwrap();
    assert_eq!(made, 1, "round-1 final must be created");

    let all = list_matches(&db, &tid).await;
    assert_eq!(all.len(), 3);
    let final_match = all.iter().find(|m| m.round == Some(1)).unwrap();
    assert_eq!(final_match.bracket.as_deref(), Some("winners"));
    assert_eq!(final_match.participants.len(), 2);

    // Finish the final.
    complete_match_with_winner(&db, final_match.id.clone().unwrap(), 0).await;
    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();
    let after = tournament::get_tournament(&db, tid).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Completed);
}

#[tokio::test]
async fn double_elim_four_players_runs_full_bracket_with_grand_final() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let players = vec![
        seeded_user(&db, &cfg.bob.email).await,
        seeded_user(&db, &cfg.alice.email).await,
        ensure_user(&db, 11).await,
        ensure_user(&db, 12).await,
    ];
    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("DE4 "),
        "double elim 4".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::DoubleElimination),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for p in &players {
        register_player(&db, &tid, p.clone()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    // WB R0: 2 matches.
    let r0 = list_matches(&db, &tid).await;
    assert_eq!(r0.len(), 2);
    // Settle WB R0: index-0 wins both. Their losers drop to LB R0.
    for m in &r0 {
        complete_match_with_winner(&db, m.id.clone().unwrap(), 0).await;
    }
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    // After advance: WB R1 (final) created + LB R0 (1 match) created.
    let after_r0 = list_matches(&db, &tid).await;
    let wb_final = after_r0
        .iter()
        .find(|m| m.bracket.as_deref() == Some("winners") && m.round == Some(1))
        .expect("WB final created");
    let lb_r0 = after_r0
        .iter()
        .find(|m| m.bracket.as_deref() == Some("losers") && m.round == Some(0))
        .expect("LB R0 created");
    assert_eq!(lb_r0.participants.len(), 2);

    // Finish WB final and LB R0.
    complete_match_with_winner(&db, wb_final.id.clone().unwrap(), 0).await;
    complete_match_with_winner(&db, lb_r0.id.clone().unwrap(), 0).await;
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    // LB R1 (drop) should now exist: WB final loser vs LB R0 winner.
    let after_r1 = list_matches(&db, &tid).await;
    let lb_r1 = after_r1
        .iter()
        .find(|m| m.bracket.as_deref() == Some("losers") && m.round == Some(1))
        .expect("LB R1 created");
    assert_eq!(lb_r1.participants.len(), 2);

    complete_match_with_winner(&db, lb_r1.id.clone().unwrap(), 0).await;
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    // Grand final must now exist: WB final winner vs LB R1 winner.
    let after_lb = list_matches(&db, &tid).await;
    let gf = after_lb
        .iter()
        .find(|m| m.bracket.as_deref() == Some("grand_final"))
        .expect("grand_final created");

    // WB-side wins -> no reset. Tournament should finalize.
    complete_match_with_winner(&db, gf.id.clone().unwrap(), 0).await;
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();
    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();

    let after = tournament::get_tournament(&db, tid.clone()).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Completed);
    let all = list_matches(&db, &tid).await;
    let reset = all
        .iter()
        .find(|m| m.bracket.as_deref() == Some("grand_final_reset"));
    assert!(reset.is_none(), "no reset when WB-side wins GF");
}

#[tokio::test]
async fn double_elim_grand_final_reset_fires_when_lb_side_wins() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let players = vec![
        seeded_user(&db, &cfg.bob.email).await,
        seeded_user(&db, &cfg.alice.email).await,
        ensure_user(&db, 21).await,
        ensure_user(&db, 22).await,
    ];
    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("DE4r "),
        "double elim 4 reset".into(),
        2,
        4,
        None,
        None,
        Some(MatchGenerationType::DoubleElimination),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for p in &players {
        register_player(&db, &tid, p.clone()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    let r0 = list_matches(&db, &tid).await;
    for m in &r0 {
        complete_match_with_winner(&db, m.id.clone().unwrap(), 0).await;
    }
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    let after_r0 = list_matches(&db, &tid).await;
    let wb_final = after_r0
        .iter()
        .find(|m| m.bracket.as_deref() == Some("winners") && m.round == Some(1))
        .unwrap();
    let lb_r0 = after_r0
        .iter()
        .find(|m| m.bracket.as_deref() == Some("losers") && m.round == Some(0))
        .unwrap();
    complete_match_with_winner(&db, wb_final.id.clone().unwrap(), 0).await;
    complete_match_with_winner(&db, lb_r0.id.clone().unwrap(), 0).await;
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    let after_r1 = list_matches(&db, &tid).await;
    let lb_r1 = after_r1
        .iter()
        .find(|m| m.bracket.as_deref() == Some("losers") && m.round == Some(1))
        .unwrap();
    complete_match_with_winner(&db, lb_r1.id.clone().unwrap(), 0).await;
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    let after_gf_made = list_matches(&db, &tid).await;
    let gf = after_gf_made
        .iter()
        .find(|m| m.bracket.as_deref() == Some("grand_final"))
        .unwrap();

    // LB-side wins (index 1) -> reset spawned.
    complete_match_with_winner(&db, gf.id.clone().unwrap(), 1).await;
    bracket::advance_brackets(&db, tid.clone()).await.unwrap();

    let with_reset = list_matches(&db, &tid).await;
    let reset = with_reset
        .iter()
        .find(|m| m.bracket.as_deref() == Some("grand_final_reset"))
        .expect("grand_final_reset must exist when LB-side wins GF");
    complete_match_with_winner(&db, reset.id.clone().unwrap(), 1).await;
    finalization::finalize_if_done(&db, tid.clone()).await.unwrap();
    let after = tournament::get_tournament(&db, tid).await.unwrap();
    assert_eq!(after.status, TournamentStatus::Completed);
}

#[tokio::test]
async fn single_elim_three_players_top_seed_gets_bye_to_round_one() {
    let db = db::setup_test_db().await;
    let cfg = Config::from_env();
    let players = vec![
        seeded_user(&db, &cfg.bob.email).await,
        seeded_user(&db, &cfg.alice.email).await,
        ensure_user(&db, 3).await,
    ];

    let t = tournament::create_tournament(
        &db,
        GAME.into(),
        unique("SE3 "),
        "single elim 3".into(),
        2,
        3,
        None,
        None,
        Some(MatchGenerationType::SingleElimination),
        Some(TournamentKind::Bot), None,
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    for p in &players {
        register_player(&db, &tid, p.clone()).await;
    }
    tournament::start_tournament(&db, tid.clone()).await.unwrap();

    let r0 = list_matches(&db, &tid).await;
    assert_eq!(r0.len(), 2, "two round-0 slots: one BYE + one real match");

    let bye = r0
        .iter()
        .find(|m| m.participants.len() == 1)
        .expect("BYE match");
    assert_eq!(bye.status, MatchStatus::Completed);

    let live = r0.iter().find(|m| m.participants.len() == 2).unwrap();
    complete_match_with_winner(&db, live.id.clone().unwrap(), 0).await;

    let made = bracket::advance_brackets(&db, tid.clone()).await.unwrap();
    assert_eq!(made, 1);
    let all = list_matches(&db, &tid).await;
    let r1 = all.iter().find(|m| m.round == Some(1)).unwrap();
    assert_eq!(r1.participants.len(), 2);
}
