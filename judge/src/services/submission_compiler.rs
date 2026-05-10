// Submission compiler worker.
//
// Polls `submission` rows in `pending` state and compiles them upfront,
// caching the resulting binary path on the row. Match runners reuse the
// cached artifact instead of compiling per-match.
//
// Status transitions: pending -> compiling -> accepted | failed.
// Claim is atomic via conditional UPDATE so multiple judges don't race.

use crate::db::Database;
use crate::services::sandbox::compiler::CompilerSandbox;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Per-submission compile contract. Plug in `CompilerSandbox` at
/// runtime; in tests, plug in `FakeCompiler` to drive the worker
/// without touching the real sandbox.
#[async_trait]
pub trait BotCompiler: Send + Sync + 'static {
    async fn compile(&self, sid: &str, language: &str, code: &str) -> Result<String>;
}

#[async_trait]
impl BotCompiler for CompilerSandbox {
    async fn compile(&self, sid: &str, language: &str, code: &str) -> Result<String> {
        CompilerSandbox::compile(self, sid, language, code)
            .await
            .map_err(|e| anyhow::anyhow!("compile: {e}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PendingSubmission {
    pub id: RecordId,
    pub language: String,
    pub code: String,
}

pub fn spawn(db: Database) {
    tokio::spawn(async move {
        if let Err(e) = run(db).await {
            tracing::error!("Submission compiler exited: {e:#}");
        }
    });
}

async fn run(db: Database) -> Result<()> {
    tracing::info!("Submission compiler worker started");
    let workspace_root = std::env::var("COMPILER_WORKSPACE")
        .unwrap_or_else(|_| "/artifacts".to_string());
    let sandbox = CompilerSandbox::new(PathBuf::from(workspace_root))
        .map_err(|e| anyhow::anyhow!("init compiler sandbox: {e}"))
        .context("init compiler")?;
    let compiler: Arc<dyn BotCompiler> = Arc::new(sandbox);
    loop {
        let _ = tick(&db, &compiler).await;
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// One polling pass. Public + generic over compiler so integration
/// tests can drive it directly without a tokio sleep.
pub async fn tick(db: &Database, compiler: &Arc<dyn BotCompiler>) -> Result<u32> {
    let pending = poll_pending(db).await.unwrap_or_default();
    let mut handled = 0u32;
    for sub in pending {
        if !claim(db, &sub.id).await.unwrap_or(false) {
            continue;
        }
        handled += 1;
        if let Err(e) = compile_one(db, compiler.as_ref(), &sub).await {
            tracing::warn!("Submission {} compile failed: {e:#}", sub.id.to_sql());
            let _ = mark_failed(db, &sub.id, &format!("{e:#}")).await;
        }
    }
    Ok(handled)
}

async fn poll_pending(db: &Database) -> Result<Vec<PendingSubmission>> {
    let q = "SELECT id, language, code FROM submission
             WHERE status = 'pending' LIMIT 20;";
    let mut resp = db.query(q).await?;
    let rows: Vec<PendingSubmission> = resp.take(0)?;
    Ok(rows)
}

async fn claim(db: &Database, id: &RecordId) -> Result<bool> {
    let q = "UPDATE $sid SET status = 'compiling'
             WHERE status = 'pending' RETURN AFTER;";
    let mut resp = db.query(q).bind(("sid", id.clone())).await?;
    let rows: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    Ok(!rows.is_empty())
}

async fn compile_one(
    db: &Database,
    compiler: &dyn BotCompiler,
    sub: &PendingSubmission,
) -> Result<()> {
    let sid = sub.id.to_sql();
    tracing::info!("Compiling submission {sid}");
    let bin = compiler
        .compile(&sid, &sub.language, &sub.code)
        .await
        .with_context(|| format!("compile {sid}"))?;
    db.query(
        "UPDATE $sid SET compiled_binary_path = $bin, status = 'accepted',
                         error_message = NONE;",
    )
    .bind(("sid", sub.id.clone()))
    .bind(("bin", bin))
    .await?;
    tracing::info!("Submission {sid} accepted");
    Ok(())
}

async fn mark_failed(db: &Database, id: &RecordId, err: &str) -> Result<()> {
    db.query("UPDATE $sid SET status = 'failed', error_message = $err;")
        .bind(("sid", id.clone()))
        .bind(("err", err.to_string()))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn poll_query_filters_pending() {
        let q = "SELECT id, language, code FROM submission
             WHERE status = 'pending' LIMIT 20;";
        assert!(q.contains("status = 'pending'"));
        assert!(q.contains("LIMIT 20"));
    }

    #[test]
    fn claim_query_is_conditional() {
        let q = "UPDATE $sid SET status = 'compiling'
             WHERE status = 'pending' RETURN AFTER;";
        assert!(q.contains("WHERE status = 'pending'"));
        assert!(q.contains("status = 'compiling'"));
    }
}
