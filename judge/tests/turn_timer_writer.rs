// Integration test: turn_timer's default db callback writes a match
// row + finishes the room when invoked.

mod db;

use judge::services::turn_timer::db_timeout_callback;
use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue, ToSql};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct UserStub {
    id: Option<RecordId>,
    email: String,
    username: String,
    role: String,
    location: String,
    is_banned: bool,
    created_at: Datetime,
    updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct RoomStub {
    id: Option<RecordId>,
    game_id: String,
    host_id: RecordId,
    name: String,
    max_players: i64,
    status: String,
    players: Vec<RecordId>,
    tournament_id: Option<RecordId>,
    allowed_user_ids: Vec<RecordId>,
    is_ranked: bool,
    winner_id: Option<RecordId>,
    event_history: Vec<String>,
    created_at: Datetime,
    updated_at: Datetime,
}

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct MatchRow {
    status: String,
    error_message: Option<String>,
    faulted_user_ids: Vec<RecordId>,
    participants: Vec<MatchPart>,
}

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct MatchPart {
    user_id: RecordId,
    score: Option<f64>,
}

fn unique(p: &str) -> String {
    format!("{p}_{}", uuid::Uuid::new_v4().simple())
}

async fn make_user(db: &judge::db::Database, name: &str) -> RecordId {
    let mut resp = db
        .query(
            "CREATE user SET email=$e, username=$u, role='player',
                          location='US', is_banned=false,
                          created_at=time::now(), updated_at=time::now()
             RETURN AFTER",
        )
        .bind(("e", format!("{name}@test.local")))
        .bind(("u", name.to_string()))
        .await
        .unwrap();
    let rows: Vec<UserStub> = resp.take(0).unwrap();
    rows.into_iter().next().unwrap().id.unwrap()
}

async fn make_room(
    db: &judge::db::Database,
    host: RecordId,
    other: RecordId,
) -> RecordId {
    let mut resp = db
        .query(
            "CREATE room SET game_id='tic-tac-toe',
                              host_id=$h,
                              name=$n,
                              max_players=2,
                              status='playing',
                              players=$ps,
                              allowed_user_ids=[],
                              is_ranked=false,
                              tournament_id=NONE,
                              winner_id=NONE,
                              event_history=[],
                              created_at=time::now(),
                              updated_at=time::now()
             RETURN AFTER",
        )
        .bind(("h", host.clone()))
        .bind(("n", unique("room-")))
        .bind(("ps", vec![host, other]))
        .await
        .unwrap();
    let rows: Vec<RoomStub> = resp.take(0).unwrap();
    rows.into_iter().next().unwrap().id.unwrap()
}

async fn fetch_room(db: &judge::db::Database, rid: &RecordId) -> RoomStub {
    let r: Option<RoomStub> = db.select(rid).await.unwrap();
    r.unwrap()
}

async fn fetch_room_match(db: &judge::db::Database, rid: &RecordId) -> Option<MatchRow> {
    let mut resp = db
        .query("SELECT status, error_message, faulted_user_ids, participants FROM match WHERE room_id = $rid")
        .bind(("rid", rid.clone()))
        .await
        .unwrap();
    let rows: Vec<MatchRow> = resp.take(0).unwrap_or_default();
    rows.into_iter().next()
}

#[tokio::test]
async fn timeout_callback_writes_match_and_finishes_room() {
    let db = db::setup_test_db().await;
    let alice = make_user(&db, &unique("alice")).await;
    let bob = make_user(&db, &unique("bob")).await;
    let rid = make_room(&db, alice.clone(), bob.clone()).await;

    let cb = db_timeout_callback(db.clone());
    // Bob is the lagger; alice should win by default.
    cb(rid.to_sql(), vec![bob.to_sql()]).await;

    let room = fetch_room(&db, &rid).await;
    assert_eq!(room.status, "finished");
    assert_eq!(room.winner_id.as_ref(), Some(&alice));

    let m = fetch_room_match(&db, &rid).await.expect("match row");
    assert_eq!(m.status, "completed");
    assert_eq!(m.error_message.as_deref(), Some("turn_timeout"));
    assert_eq!(m.faulted_user_ids, vec![bob.clone()]);
    let bob_score = m
        .participants
        .iter()
        .find(|p| p.user_id == bob)
        .and_then(|p| p.score);
    let alice_score = m
        .participants
        .iter()
        .find(|p| p.user_id == alice)
        .and_then(|p| p.score);
    assert_eq!(bob_score, Some(0.0));
    assert_eq!(alice_score, Some(1.0));
}

#[tokio::test]
async fn timeout_callback_no_op_when_room_already_finished() {
    let db = db::setup_test_db().await;
    let alice = make_user(&db, &unique("alice")).await;
    let bob = make_user(&db, &unique("bob")).await;
    let rid = make_room(&db, alice.clone(), bob.clone()).await;
    db.query("UPDATE $rid SET status='finished'")
        .bind(("rid", rid.clone()))
        .await
        .unwrap();

    let cb = db_timeout_callback(db.clone());
    cb(rid.to_sql(), vec![bob.to_sql()]).await;

    // No match row should have been created.
    assert!(fetch_room_match(&db, &rid).await.is_none());
}

#[tokio::test]
async fn timeout_callback_with_unknown_room_is_noop() {
    let db = db::setup_test_db().await;
    let cb = db_timeout_callback(db.clone());
    // Should not panic / return error.
    cb("room:does_not_exist".into(), vec!["someone".into()]).await;
}
