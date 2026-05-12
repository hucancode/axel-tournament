// Submission worker.
//
// Polls `submission` rows in `pending` state and compiles them upfront,
// caching the resulting binary path on the row. Match runners reuse
// the cached artifact instead of compiling per-match.
//
// Status transitions: pending -> compiling -> accepted | failed.

use crate::db::Database;
use crate::models::PendingSubmission;
use crate::services::sandbox::BuildSandbox;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Fallback poll cadence. LIVE query drives the worker for instant
/// reaction; this interval only catches anything LIVE missed (cold
/// start, transient disconnect, dropped notification).
const POLL_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct SubmissionTick {
    #[serde(default)]
    id: Option<RecordId>,
}

/// Compile contract. Tests substitute a fake; production wires up
/// `BuildSandbox`.
#[async_trait]
pub trait BotCompiler: Send + Sync {
    async fn compile(&self, sid: &str, language: &str, code: &str) -> Result<String>;
}

#[async_trait]
impl BotCompiler for BuildSandbox {
    async fn compile(&self, sid: &str, language: &str, code: &str) -> Result<String> {
        BuildSandbox::compile(self, sid, language, code)
            .await
            .map_err(Into::into)
    }
}

pub fn spawn(db: Database, compiler: Arc<dyn BotCompiler>) {
    tokio::spawn(async move {
        if let Err(e) = run(db, compiler).await {
            tracing::error!("Submission worker exited: {e:#}");
        }
    });
}

async fn run(db: Database, compiler: Arc<dyn BotCompiler>) -> Result<()> {
    tracing::info!("Submission worker started");
    let _ = tick(&db, compiler.as_ref()).await;
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        let stream_res = db.select::<Vec<SubmissionTick>>("submission").live().await;
        match stream_res {
            Ok(mut stream) => {
                loop {
                    tokio::select! {
                        biased;
                        notify = stream.next() => {
                            match notify {
                                Some(Ok(_)) => {
                                    let _ = tick(&db, compiler.as_ref()).await;
                                }
                                Some(Err(e)) => {
                                    tracing::warn!("LIVE stream error: {e:#}; resubscribing");
                                    break;
                                }
                                None => {
                                    tracing::warn!("LIVE stream closed; resubscribing");
                                    break;
                                }
                            }
                        }
                        _ = ticker.tick() => {
                            let _ = tick(&db, compiler.as_ref()).await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("LIVE subscribe failed: {e:#}; falling back to poll");
                ticker.tick().await;
                let _ = tick(&db, compiler.as_ref()).await;
            }
        }
    }
}

/// One polling pass. Generic over the compiler so integration tests
/// can drive it without a real toolchain.
pub async fn tick<C: BotCompiler + ?Sized>(db: &Database, compiler: &C) -> Result<u32> {
    let pending = poll_pending(db).await.unwrap_or_default();
    let mut handled = 0u32;
    for sub in pending {
        if !claim(db, &sub.id).await.unwrap_or(false) {
            continue;
        }
        handled += 1;
        if let Err(e) = compile_one(db, compiler, &sub).await {
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

async fn compile_one<C: BotCompiler + ?Sized>(
    db: &Database,
    compiler: &C,
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
