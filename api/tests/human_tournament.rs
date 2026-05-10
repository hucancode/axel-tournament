// Human tournament end-to-end coverage.
//
// Maps to the spec phases:
//   * Join tournament + create room manually (unranked).
//   * System auto-matchmakes registered players into ranked rooms.
//   * Other players join (allowed-list enforced for ranked).
//   * Host starts game; non-host start rejected.
//   * Disconnect-timeout finishes room with the survivor as winner.
//   * Ranked match feeds ELO and tournament finalization.

mod db;

use api::{
    config::Config,
    models::{TournamentKind, TournamentStatus, RoomStatus},
    services::{auth, matchmaking, room as room_svc, tournament},
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

async fn bob(db: &api::db::Database) -> RecordId {
    let cfg = Config::from_env();
    auth::get_user_by_email(db, &cfg.bob.email)
        .await
        .unwrap()
        .unwrap()
        .id
        .unwrap()
}

async fn alice(db: &api::db::Database) -> RecordId {
    let cfg = Config::from_env();
    auth::get_user_by_email(db, &cfg.alice.email)
        .await
        .unwrap()
        .unwrap()
        .id
        .unwrap()
}

async fn human_tournament(db: &api::db::Database, status: TournamentStatus) -> RecordId {
    let t = tournament::create_tournament(
        db,
        GAME.to_string(),
        unique("Human Tour "),
        "live".into(),
        2,
        16,
        None,
        None,
        None,
        Some(TournamentKind::Human),
    )
    .await
    .unwrap();
    let tid = t.id.unwrap();
    if status != TournamentStatus::Registration {
        let st = match status {
            TournamentStatus::Registration => "registration",
            TournamentStatus::Generating => "generating",
            TournamentStatus::Running => "running",
            TournamentStatus::Completed => "completed",
            TournamentStatus::Cancelled => "cancelled",
            TournamentStatus::Scheduled => "scheduled",
        };
        db.query("UPDATE $tid SET status = $st")
            .bind(("tid", tid.clone()))
            .bind(("st", st.to_string()))
            .await
            .unwrap();
    }
    tid
}

#[tokio::test]
async fn user_creates_unranked_room_then_other_joins() {
    let db = db::setup_test_db().await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;

    let room = room_svc::create_unranked_room_for_user(
        &db,
        bob.clone(),
        GAME.into(),
        unique("UR "),
        4,
    )
    .await
    .unwrap();
    let rid = room.id.clone().unwrap();

    let after = room_svc::join_room(&db, rid, alice.clone()).await.unwrap();
    assert!(after.players.contains(&bob));
    assert!(after.players.contains(&alice));
    assert_eq!(after.status, RoomStatus::Lobby);
    assert!(!after.is_ranked);
}

#[tokio::test]
async fn ranked_room_blocks_uninvited_user() {
    let db = db::setup_test_db().await;
    let tid = human_tournament(&db, TournamentStatus::Running).await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;

    let room = room_svc::create_ranked_room(&db, tid, vec![bob.clone()]).await;
    // create_ranked_room demands >=2 allowed; expect failure on a single-allowed list
    assert!(room.is_err());

    let tid = human_tournament(&db, TournamentStatus::Running).await;
    let room = room_svc::create_ranked_room(&db, tid, vec![bob.clone(), alice.clone()])
        .await
        .unwrap();
    let rid = room.id.unwrap();

    // Anonymous user not in allowed_user_ids: forbidden.
    let stranger = RecordId::parse_simple("user:stranger").unwrap();
    let r = room_svc::join_room(&db, rid.clone(), stranger).await;
    assert!(r.is_err(), "uninvited user must be blocked");

    // Allowed users get in.
    room_svc::join_room(&db, rid.clone(), alice.clone()).await.unwrap();
    let after = room_svc::get_room(&db, rid).await.unwrap();
    assert!(after.players.contains(&alice));
    assert!(after.is_ranked);
}

#[tokio::test]
async fn matchmaker_pairs_two_queued_players_into_ranked_room() {
    let db = db::setup_test_db().await;
    let tid = human_tournament(&db, TournamentStatus::Registration).await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;

    tournament::join_tournament(&db, tid.clone(), bob.clone()).await.unwrap();
    tournament::join_tournament(&db, tid.clone(), alice.clone()).await.unwrap();

    db.query("UPDATE $tid SET status = 'running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    matchmaking::enqueue(&db, tid.clone(), bob.clone()).await.unwrap();
    matchmaking::enqueue(&db, tid.clone(), alice.clone()).await.unwrap();

    let paired = matchmaking::tick(&db, tid.clone()).await.unwrap();
    assert_eq!(paired, 1);

    let mut resp = db
        .query("SELECT * FROM room WHERE tournament_id = $tid")
        .bind(("tid", tid))
        .await
        .unwrap();
    let rooms: Vec<api::models::room::Room> = resp.take(0).unwrap();
    assert_eq!(rooms.len(), 1);
    let r = &rooms[0];
    assert!(r.is_ranked);
    assert!(r.allowed_user_ids.contains(&bob));
    assert!(r.allowed_user_ids.contains(&alice));
}

#[tokio::test]
async fn only_host_can_start_room() {
    let db = db::setup_test_db().await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;

    let room = room_svc::create_unranked_room_for_user(
        &db,
        bob.clone(),
        GAME.into(),
        unique("Start "),
        4,
    )
    .await
    .unwrap();
    let rid = room.id.unwrap();
    room_svc::join_room(&db, rid.clone(), alice.clone()).await.unwrap();

    // Non-host start: forbidden.
    let r = room_svc::start_room(&db, rid.clone(), alice).await;
    assert!(r.is_err());

    // Host start: works.
    let started = room_svc::start_room(&db, rid, bob).await.unwrap();
    assert_eq!(started.status, RoomStatus::Playing);
}

#[tokio::test]
async fn start_room_requires_at_least_two_players() {
    let db = db::setup_test_db().await;
    let bob = bob(&db).await;
    let room = room_svc::create_unranked_room_for_user(
        &db,
        bob.clone(),
        GAME.into(),
        unique("Lonely "),
        4,
    )
    .await
    .unwrap();
    let r = room_svc::start_room(&db, room.id.unwrap(), bob).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn finish_room_disconnect_timeout_makes_other_player_winner() {
    let db = db::setup_test_db().await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;
    let room = room_svc::create_unranked_room_for_user(
        &db,
        bob.clone(),
        GAME.into(),
        unique("DC "),
        2,
    )
    .await
    .unwrap();
    let rid = room.id.unwrap();
    room_svc::join_room(&db, rid.clone(), alice.clone()).await.unwrap();
    room_svc::start_room(&db, rid.clone(), bob.clone()).await.unwrap();

    // Alice disconnects without returning to make her move: she
    // timed out, so she is the faulted player. Bob is winner-by-default.
    let finished = room_svc::finish_room(
        &db,
        rid.clone(),
        Some(bob.clone()),
        room_svc::FinishReason::DisconnectTimeout,
        vec![alice.clone()],
    )
    .await
    .unwrap();

    assert_eq!(finished.status, RoomStatus::Finished);
    assert_eq!(finished.winner_id.as_ref(), Some(&bob));

    // A match row was written with disconnect_timeout reason.
    let mut resp = db
        .query("SELECT * FROM match WHERE room_id = $rid")
        .bind(("rid", rid))
        .await
        .unwrap();
    let ms: Vec<api::models::matches::Match> = resp.take(0).unwrap();
    let m = &ms[0];
    assert_eq!(m.error_message.as_deref(), Some("disconnect_timeout"));
}

#[tokio::test]
async fn ranked_finish_updates_elo_for_both_players() {
    let db = db::setup_test_db().await;
    let tid = human_tournament(&db, TournamentStatus::Registration).await;
    let bob = bob(&db).await;
    let alice = alice(&db).await;
    tournament::join_tournament(&db, tid.clone(), bob.clone()).await.unwrap();
    tournament::join_tournament(&db, tid.clone(), alice.clone()).await.unwrap();
    db.query("UPDATE $tid SET status = 'running'")
        .bind(("tid", tid.clone()))
        .await
        .unwrap();

    let room = room_svc::create_ranked_room(
        &db,
        tid.clone(),
        vec![bob.clone(), alice.clone()],
    )
    .await
    .unwrap();
    let rid = room.id.unwrap();
    room_svc::join_room(&db, rid.clone(), alice.clone()).await.unwrap();
    room_svc::start_room(&db, rid.clone(), bob.clone()).await.unwrap();

    let finished = room_svc::finish_room(
        &db,
        rid,
        Some(bob.clone()),
        room_svc::FinishReason::Played,
        Vec::new(),
    )
    .await
    .unwrap();
    assert_eq!(finished.status, RoomStatus::Finished);

    let parts = tournament::get_tournament_participants(&db, tid).await.unwrap();
    let bob_p = parts.iter().find(|p| p.user_id == bob).unwrap();
    let alice_p = parts.iter().find(|p| p.user_id == alice).unwrap();
    let bob_elo = bob_p.elo.expect("bob elo set after ranked match");
    let alice_elo = alice_p.elo.expect("alice elo set after ranked match");
    assert!(bob_elo > 1000.0, "winner ELO above default");
    assert!(alice_elo < 1000.0, "loser ELO below default");
    assert!((bob_elo - 1000.0 + alice_elo - 1000.0).abs() < 0.01,
        "K-factor: gain == loss");
}
