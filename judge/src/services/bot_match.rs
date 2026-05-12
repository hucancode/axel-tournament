// Bot-vs-bot tournament runner.
//
// Same wire protocol + same RoomLogic + same EventLog as human rooms.
// One match = one room. Bots speak the stdio transport described in
// `judge/protocols/wire.md`.
//
// Per-game dispatch is via `HashMap<game_id, Arc<dyn BotMatchHost>>`
// — adding a new game means registering an impl, not editing this file.

use crate::db::Database;
use crate::models::{Match, MatchParticipant, Submission};
use crate::services::capacity::CapacityTracker;
use crate::services::room::bot::{run_match, wrap, Bot, MatchOutcome};
use crate::services::room::logic::{RoomLogic, RoomRegistry};
use crate::services::sandbox::SandboxedBot;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::types::SurrealValue;
use std::time::Duration;
use surrealdb::types::{RecordId, ToSql};

/// Fallback poll cadence. LIVE drives the watcher in the steady state;
/// this interval only catches reconnect gaps.
const POLL_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct MatchTick {
    #[serde(default)]
    id: Option<surrealdb::types::RecordId>,
}
const LEASE_TTL: Duration = Duration::from_secs(15);

/// Erasure over per-game `RoomRegistry<L>`. Lets the claim loop run any
/// registered game without knowing its concrete `L`. Implementations
/// open the room, run the match, drop the room — the trait owns the
/// full lease lifecycle so we can't leak on error.
#[async_trait]
pub trait BotMatchHost: Send + Sync + 'static {
    async fn run(
        &self,
        match_id: &str,
        bots: Vec<Bot>,
        player_ids: Vec<String>,
        turn_timeout: Duration,
    ) -> Result<MatchOutcome>;
}

#[async_trait]
impl<L: RoomLogic> BotMatchHost for Arc<RoomRegistry<L>> {
    async fn run(
        &self,
        match_id: &str,
        bots: Vec<Bot>,
        player_ids: Vec<String>,
        turn_timeout: Duration,
    ) -> Result<MatchOutcome> {
        let room = self.open(match_id, LEASE_TTL).await?;
        let outcome = run_match(room, bots, player_ids, turn_timeout).await;
        // Drop the room regardless of outcome: lease released, in-memory
        // state evicted. Log remains for audit / replay.
        RoomRegistry::drop_room(self, match_id).await;
        outcome
    }
}

/// Map of `game_id` → host. The watcher dispatches on `Match::game_id`.
pub type BotMatchRegistries = HashMap<&'static str, Arc<dyn BotMatchHost>>;

pub fn spawn(db: Database, capacity: CapacityTracker, registries: BotMatchRegistries) {
    tokio::spawn(async move {
        if let Err(e) = run(db, capacity, registries).await {
            tracing::error!("bot match watcher exited: {e:#}");
        }
    });
}

async fn run(
    db: Database,
    capacity: CapacityTracker,
    registries: BotMatchRegistries,
) -> Result<()> {
    tracing::info!("bot match watcher started");
    drain_once(&db, &capacity, &registries).await;
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        let stream_res = db.select::<Vec<MatchTick>>("match").live().await;
        match stream_res {
            Ok(mut stream) => loop {
                tokio::select! {
                    biased;
                    notify = stream.next() => {
                        match notify {
                            Some(Ok(_)) => drain_once(&db, &capacity, &registries).await,
                            Some(Err(e)) => {
                                tracing::warn!("match LIVE error: {e:#}; resubscribing");
                                break;
                            }
                            None => {
                                tracing::warn!("match LIVE closed; resubscribing");
                                break;
                            }
                        }
                    }
                    _ = ticker.tick() => {
                        drain_once(&db, &capacity, &registries).await;
                    }
                }
            },
            Err(e) => {
                tracing::warn!("match LIVE subscribe failed: {e:#}; falling back to poll");
                ticker.tick().await;
                drain_once(&db, &capacity, &registries).await;
            }
        }
    }
}

async fn drain_once(
    db: &Database,
    capacity: &CapacityTracker,
    registries: &BotMatchRegistries,
) {
    let matches = poll_pending(db).await.unwrap_or_else(|e| {
        tracing::error!("poll pending matches: {e}");
        Vec::new()
    });
    for m in matches {
        if !capacity.can_accept_work() {
            break;
        }
        let delay_ms = capacity.claim_delay_ms();
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        if !claim(db, &m.id).await.unwrap_or(false) {
            continue;
        }
        tracing::info!("Claimed match: {}", m.id.to_sql());
        let slot = capacity.match_slot();

        let db_c = db.clone();
        let regs = registries.clone();
        tokio::spawn(async move {
            let mid = m.id.clone();
            if let Err(e) = execute(&db_c, &regs, m).await {
                tracing::error!("Match {} failed: {e:#}", mid.to_sql());
                let _ = mark_failed(&db_c, &mid, &e.to_string()).await;
            }
            drop(slot);
        });
    }
}

async fn poll_pending(db: &Database) -> Result<Vec<Match>> {
    let q = "SELECT * FROM match
             WHERE status = 'pending' AND tournament_id != NONE
             ORDER BY created_at LIMIT 10;";
    let mut resp = db.query(q).await.context("query pending")?;
    let rows: Vec<Match> = resp.take(0).context("decode pending")?;
    Ok(rows)
}

async fn claim(db: &Database, mid: &RecordId) -> Result<bool> {
    let q = "UPDATE $mid SET status = 'queued', updated_at = time::now()
             WHERE status = 'pending' RETURN AFTER;";
    let mut resp = db.query(q).bind(("mid", mid.clone())).await?;
    let rows: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    Ok(!rows.is_empty())
}

async fn mark_running(db: &Database, mid: &RecordId) -> Result<()> {
    db.query(
        "UPDATE $mid SET status = 'running',
                          started_at = time::now(),
                          updated_at = time::now();",
    )
    .bind(("mid", mid.clone()))
    .await?;
    Ok(())
}

async fn execute(db: &Database, regs: &BotMatchRegistries, m: Match) -> Result<()> {
    let id_str = m.id.to_sql();
    mark_running(db, &m.id).await?;

    let metadata = crate::games::find_game_by_id(&m.game_id)
        .ok_or_else(|| anyhow!("unknown game_id: {}", m.game_id))?;
    let turn_timeout = Duration::from_millis(metadata.bot_turn_timeout_ms);

    let host = regs
        .get(m.game_id.as_str())
        .ok_or_else(|| anyhow!("unsupported game: {}", m.game_id))?
        .clone();

    let binaries = resolve_binaries(db, &m.participants).await?;
    let player_ids: Vec<String> = m
        .participants
        .iter()
        .map(|p| p.submission_id.to_sql())
        .collect();

    let mut bots: Vec<Bot> = Vec::with_capacity(binaries.len());
    for (pid, bin) in player_ids.iter().zip(&binaries) {
        let sb = SandboxedBot::spawn(pid, bin)
            .await
            .with_context(|| format!("spawn bot {pid}"))?;
        bots.push(wrap(sb));
    }

    let outcome = host.run(&id_str, bots, player_ids, turn_timeout).await?;
    write_scores(db, &m, outcome).await?;
    Ok(())
}

/// Resolve every participant's compiled binary path. By the time a
/// match reaches the runner, `start_tournament` has already filtered
/// out participants whose submission is not in the `accepted` state —
/// missing or failed submissions here are a data-integrity bug.
async fn resolve_binaries(
    db: &Database,
    participants: &[MatchParticipant],
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::with_capacity(participants.len());
    for p in participants {
        let sid = p.submission_id.to_sql();
        let mut resp = db
            .query(
                "SELECT compiled_binary_path, language, code, status, error_message
                 FROM $sid;",
            )
            .bind(("sid", p.submission_id.clone()))
            .await
            .with_context(|| format!("query submission {sid}"))?;
        let rows: Vec<Submission> = resp.take(0)?;
        let s = rows
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("submission {sid} missing"))?;
        if s.status.as_deref() != Some("accepted") {
            return Err(anyhow!(
                "submission {sid} not accepted (status={:?})",
                s.status
            ));
        }
        let path = s
            .compiled_binary_path
            .filter(|p| !p.is_empty())
            .ok_or_else(|| anyhow!("submission {sid} accepted but no binary path"))?;
        paths.push(PathBuf::from(path));
    }
    Ok(paths)
}

async fn write_scores(db: &Database, m: &Match, outcome: MatchOutcome) -> Result<()> {
    let s0 = outcome.scores.first().copied().unwrap_or(0.0);
    let s1 = outcome.scores.get(1).copied().unwrap_or(0.0);
    let faulted: Vec<RecordId> = outcome
        .faulted_indices
        .iter()
        .filter_map(|i| m.participants.get(*i))
        .filter_map(|p| p.user_id.clone())
        .collect();
    let status = if faulted.is_empty() { "completed" } else { "failed" };
    db.query(
        "UPDATE $mid SET
             status = $status,
             participants[0].score = $s0,
             participants[1].score = $s1,
             error_message = $err,
             faulted_user_ids = $faulted,
             completed_at = time::now(),
             updated_at = time::now();",
    )
    .bind(("mid", m.id.clone()))
    .bind(("s0", s0))
    .bind(("s1", s1))
    .bind(("status", status.to_string()))
    .bind(("err", outcome.fault_reason))
    .bind(("faulted", faulted))
    .await?;
    tracing::info!(
        "Match {} -> {}: scores [{}, {}]",
        m.id.to_sql(),
        status,
        s0,
        s1
    );
    Ok(())
}

async fn mark_failed(db: &Database, mid: &RecordId, err: &str) -> Result<()> {
    db.query(
        "UPDATE $mid SET status = 'failed', error_message = $err,
                          completed_at = time::now(), updated_at = time::now();",
    )
    .bind(("mid", mid.clone()))
    .bind(("err", err.to_string()))
    .await?;
    Ok(())
}
