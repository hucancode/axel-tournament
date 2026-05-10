// Integration test: judge `submission::tick` claims a pending row and
// writes the compiled artifact path back. Uses a fake compiler so the
// test doesn't need a real toolchain.

mod db;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use judge::services::submission::{tick, BotCompiler};
use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};
use tokio::sync::Mutex;
use std::sync::OnceLock;

/// `tick` polls every pending submission in the DB. Tests must run
/// serially or they consume each other's seed rows.
fn lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct Submission {
    id: Option<RecordId>,
    user_id: RecordId,
    tournament_id: RecordId,
    game_id: String,
    language: String,
    code: String,
    status: String,
    error_message: Option<String>,
    compiled_binary_path: Option<String>,
    created_at: Datetime,
}

struct AlwaysOk(&'static str);

#[async_trait]
impl BotCompiler for AlwaysOk {
    async fn compile(&self, _sid: &str, _lang: &str, _code: &str) -> Result<String> {
        Ok(self.0.to_string())
    }
}

struct AlwaysFail;

#[async_trait]
impl BotCompiler for AlwaysFail {
    async fn compile(&self, _sid: &str, _lang: &str, _code: &str) -> Result<String> {
        Err(anyhow!("syntax error: expected `;` at line 1"))
    }
}

fn unique(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

async fn seed_pending(
    db: &judge::db::Database,
    code: &str,
) -> RecordId {
    let user_id = unique("user");
    let tournament_id = unique("tournament");
    let q = "
        CREATE submission SET
            user_id = type::record('user', $u),
            tournament_id = type::record('tournament', $t),
            game_id = 'rock-paper-scissors',
            language = 'rust',
            code = $code,
            status = 'pending',
            error_message = NONE,
            compiled_binary_path = NONE,
            created_at = time::now()
        RETURN AFTER;
    ";
    let mut resp = db
        .query(q)
        .bind(("u", user_id))
        .bind(("t", tournament_id))
        .bind(("code", code.to_string()))
        .await
        .unwrap();
    let rows: Vec<Submission> = resp.take(0).unwrap();
    rows.into_iter().next().unwrap().id.unwrap()
}

async fn fetch(db: &judge::db::Database, id: &RecordId) -> Submission {
    let s: Option<Submission> = db.select(id).await.unwrap();
    s.unwrap()
}

#[tokio::test]
async fn pending_submission_compiles_and_is_marked_accepted() {
    let _g = lock().lock().await;
    let db = db::setup_test_db().await;
    db.query("DELETE submission WHERE status = 'pending' OR status = 'compiling'")
        .await
        .unwrap();
    let sid = seed_pending(&db, "fn main() {}").await;

    let comp = AlwaysOk("/artifacts/p");
    let handled = tick(&db, &comp).await.unwrap();
    assert!(handled >= 1);

    let after = fetch(&db, &sid).await;
    assert_eq!(after.status, "accepted");
    assert_eq!(after.compiled_binary_path.as_deref(), Some("/artifacts/p"));
    assert!(after.error_message.is_none());
}

#[tokio::test]
async fn compile_failure_marks_submission_failed_with_message() {
    let _g = lock().lock().await;
    let db = db::setup_test_db().await;
    db.query("DELETE submission WHERE status = 'pending' OR status = 'compiling'")
        .await
        .unwrap();
    let sid = seed_pending(&db, "this is not rust").await;

    let comp = AlwaysFail;
    tick(&db, &comp).await.unwrap();

    let after = fetch(&db, &sid).await;
    assert_eq!(after.status, "failed");
    assert!(
        after
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("syntax error"),
        "must surface compiler diagnostics: {:?}",
        after.error_message
    );
    assert!(after.compiled_binary_path.is_none());
}

#[tokio::test]
async fn already_accepted_submission_is_not_reclaimed() {
    let _g = lock().lock().await;
    let db = db::setup_test_db().await;
    db.query("DELETE submission WHERE status = 'pending' OR status = 'compiling'")
        .await
        .unwrap();
    let sid = seed_pending(&db, "fn main() {}").await;

    // First tick: succeeds, status -> accepted.
    let comp = AlwaysOk("/artifacts/cached");
    tick(&db, &comp).await.unwrap();

    // Second tick with a different binary: must NOT touch the row,
    // since status is no longer 'pending'.
    let comp2 = AlwaysOk("/artifacts/clobbered");
    tick(&db, &comp2).await.unwrap();

    let after = fetch(&db, &sid).await;
    assert_eq!(after.compiled_binary_path.as_deref(), Some("/artifacts/cached"));
}

/// Two ticks racing on the same pending row. Exactly one claim wins;
/// the other returns 0 handled. Verified with a shared counter on the
/// fake compiler: it must be called once.
struct CountingOk {
    bin: &'static str,
    runs: std::sync::Arc<std::sync::atomic::AtomicU32>,
    delay_ms: u64,
}

#[async_trait]
impl BotCompiler for CountingOk {
    async fn compile(&self, _sid: &str, _l: &str, _c: &str) -> Result<String> {
        self.runs.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        }
        Ok(self.bin.to_string())
    }
}

#[tokio::test]
async fn concurrent_ticks_only_compile_once() {
    use std::sync::atomic::Ordering;
    use std::sync::Arc as StdArc;

    let _g = lock().lock().await;
    let db = db::setup_test_db().await;
    db.query("DELETE submission WHERE status = 'pending' OR status = 'compiling'")
        .await
        .unwrap();
    let _sid = seed_pending(&db, "fn main() {}").await;

    let runs = StdArc::new(std::sync::atomic::AtomicU32::new(0));
    let comp_a = CountingOk {
        bin: "/artifacts/a",
        runs: runs.clone(),
        delay_ms: 200,
    };
    let comp_b = CountingOk {
        bin: "/artifacts/b",
        runs: runs.clone(),
        delay_ms: 200,
    };

    let db_a = db.clone();
    let db_b = db.clone();
    let (a, b) = tokio::join!(
        async move { tick(&db_a, &comp_a).await.unwrap() },
        async move { tick(&db_b, &comp_b).await.unwrap() },
    );

    assert_eq!(a + b, 1, "exactly one tick wins the claim");
    assert_eq!(
        runs.load(Ordering::SeqCst),
        1,
        "compile fn must run exactly once"
    );
}

#[tokio::test]
async fn pending_row_appearing_after_one_tick_is_picked_up_on_next_tick() {
    let _g = lock().lock().await;
    let db = db::setup_test_db().await;
    db.query("DELETE submission WHERE status = 'pending' OR status = 'compiling'")
        .await
        .unwrap();
    let comp = AlwaysOk("/artifacts/x");

    // First tick on empty state: nobody to compile.
    let n0 = tick(&db, &comp).await.unwrap();
    assert_eq!(n0, 0);

    // Now insert a pending row and tick again.
    let sid = seed_pending(&db, "fn main() {}").await;
    let n1 = tick(&db, &comp).await.unwrap();
    assert!(n1 >= 1);

    let after = fetch(&db, &sid).await;
    assert_eq!(after.status, "accepted");
}
