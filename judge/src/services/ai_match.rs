// AI-vs-AI tournament runner.
//
// Same wire protocol + same RoomLogic + same EventLog as human rooms.
// One match = one room. Bots speak the stdio transport described in
// `judge/protocols/wire.md`. State, replay, and failover behave
// identically to human rooms.

use crate::games::{Pd, Rps, Ttt};
use crate::services::capacity::CapacityTracker;
use crate::services::compiler::Compiler;
use crate::services::room::bot::{run_match, BotConn, MatchOutcome};
use crate::services::room_logic::{LiveRoom, RoomLogic, RoomRegistry};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::{Datetime, RecordId, SurrealValue, ToSql};

use crate::db::Database;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const LEASE_TTL: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct Match {
    id: RecordId,
    tournament_id: RecordId,
    game_id: String,
    status: String,
    participants: Vec<MatchParticipant>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
    #[serde(default)]
    room_id: Option<RecordId>,
    created_at: Datetime,
    updated_at: Datetime,
    #[serde(default)]
    started_at: Option<Datetime>,
    #[serde(default)]
    completed_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct MatchParticipant {
    #[serde(default)]
    user_id: Option<RecordId>,
    submission_id: RecordId,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct Submission {
    #[serde(default)]
    id: Option<RecordId>,
    #[serde(default)]
    compiled_binary_path: Option<String>,
    language: String,
    code: String,
}

/// Per-game registry handles bundled so the claim loop can dispatch
/// on `game_id` without knowing the per-game type. The same registry
/// instance is shared with the WebSocket handler so AI and human
/// rooms ride the exact same state.
#[derive(Clone)]
pub struct AiRegistries {
    pub rps: Arc<RoomRegistry<Rps>>,
    pub ttt: Arc<RoomRegistry<Ttt>>,
    pub pd: Arc<RoomRegistry<Pd>>,
}

/// Spawn the AI match watcher loop. Polls for pending matches across
/// all known games and runs them through the same pipeline as human
/// rooms.
pub fn spawn(db: Database, capacity: CapacityTracker, registries: AiRegistries) {
    tokio::spawn(async move {
        if let Err(e) = run(db, capacity, registries).await {
            tracing::error!("AI match watcher exited: {e:#}");
        }
    });
}

async fn run(
    db: Database,
    capacity: CapacityTracker,
    registries: AiRegistries,
) -> Result<()> {
    tracing::info!("AI match watcher started");
    loop {
        let matches = poll_pending(&db).await.unwrap_or_else(|e| {
            tracing::error!("poll pending matches: {e}");
            Vec::new()
        });

        for m in matches {
            if !capacity.can_accept_work().await {
                break;
            }
            let delay_ms = capacity.calculate_claim_delay().await;
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            if !claim(&db, &m.id).await.unwrap_or(false) {
                continue;
            }
            tracing::info!("Claimed match: {}", m.id.to_sql());
            capacity.increment_matches().await;

            let db_c = db.clone();
            let cap_c = capacity.clone();
            let regs = registries.clone();
            tokio::spawn(async move {
                let id_str = m.id.to_sql();
                if let Err(e) = execute(&db_c, &regs, m).await {
                    tracing::error!("Match {id_str} failed: {e:#}");
                    let _ = mark_failed(&db_c, &id_str, &e.to_string()).await;
                }
                cap_c.decrement_matches().await;
            });
        }

        tokio::time::sleep(POLL_INTERVAL).await;
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

async fn claim(db: &Database, match_id: &RecordId) -> Result<bool> {
    let q = "UPDATE $mid SET status = 'queued', updated_at = time::now()
             WHERE status = 'pending' RETURN AFTER;";
    let mut resp = db.query(q).bind(("mid", match_id.clone())).await?;
    let rows: Vec<Match> = resp.take(0)?;
    Ok(!rows.is_empty())
}

async fn execute(db: &Database, regs: &AiRegistries, m: Match) -> Result<()> {
    let id_str = m.id.to_sql();
    db.query("UPDATE $mid SET status = 'running', started_at = time::now(), updated_at = time::now();")
        .bind(("mid", m.id.clone()))
        .await?;

    let metadata = crate::games::find_game_by_id(&m.game_id)
        .ok_or_else(|| anyhow!("unknown game_id: {}", m.game_id))?;
    let turn_timeout = Duration::from_millis(metadata.bot_turn_timeout_ms);

    let binaries = compile_all(db, &m.participants).await?;
    if binaries.len() != m.participants.len() {
        return Err(anyhow!("binary count mismatch"));
    }

    let player_ids: Vec<String> = m
        .participants
        .iter()
        .map(|p| p.submission_id.to_sql())
        .collect();

    let mut bots: Vec<Arc<BotConn>> = Vec::with_capacity(binaries.len());
    for (pid, bin) in player_ids.iter().zip(&binaries) {
        let conn = BotConn::spawn(pid, bin)
            .await
            .with_context(|| format!("spawn bot {pid}"))?;
        bots.push(Arc::new(conn));
    }

    let outcome = match m.game_id.as_str() {
        "rock-paper-scissors" => {
            let room = regs.rps.open(&id_str, LEASE_TTL).await?;
            run_one(room, bots, player_ids, turn_timeout).await
        }
        "tic-tac-toe" => {
            let room = regs.ttt.open(&id_str, LEASE_TTL).await?;
            run_one(room, bots, player_ids, turn_timeout).await
        }
        "prisoners-dilemma" => {
            let room = regs.pd.open(&id_str, LEASE_TTL).await?;
            run_one(room, bots, player_ids, turn_timeout).await
        }
        other => return Err(anyhow!("unsupported game: {other}")),
    }?;

    write_scores(db, &m, outcome).await?;

    // Drop the room: lease released, in-memory state evicted. Log
    // remains for audit / replay.
    match m.game_id.as_str() {
        "rock-paper-scissors" => regs.rps.drop_room(&id_str).await,
        "tic-tac-toe" => regs.ttt.drop_room(&id_str).await,
        "prisoners-dilemma" => regs.pd.drop_room(&id_str).await,
        _ => {}
    }
    Ok(())
}

async fn run_one<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    bots: Vec<Arc<BotConn>>,
    player_ids: Vec<String>,
    turn_timeout: Duration,
) -> Result<MatchOutcome> {
    run_match(room, bots, player_ids, turn_timeout).await
}

async fn compile_all(db: &Database, participants: &[MatchParticipant]) -> Result<Vec<PathBuf>> {
    let compiler = Compiler::new()?;
    let mut paths = Vec::with_capacity(participants.len());
    for p in participants {
        let sid = p.submission_id.to_sql();
        let mut resp = db
            .query("SELECT compiled_binary_path, language, code FROM $sid;")
            .bind(("sid", p.submission_id.clone()))
            .await
            .with_context(|| format!("query submission {sid}"))?;
        let rows: Vec<Submission> = resp.take(0)?;
        let s = rows.into_iter().next().ok_or_else(|| anyhow!("submission {sid} missing"))?;
        let path = match s.compiled_binary_path {
            Some(p) if !p.is_empty() => p,
            _ => {
                tracing::info!("Compiling submission {sid}");
                let compiled = compiler
                    .compile_submission(&sid, &s.language, &s.code)
                    .await
                    .with_context(|| format!("compile {sid}"))?;
                db.query(
                    "UPDATE $sid SET compiled_binary_path = $bin, status = 'accepted';",
                )
                .bind(("sid", p.submission_id.clone()))
                .bind(("bin", compiled.clone()))
                .await?;
                compiled
            }
        };
        paths.push(PathBuf::from(path));
    }
    Ok(paths)
}

async fn write_scores(db: &Database, m: &Match, outcome: MatchOutcome) -> Result<()> {
    let s0 = outcome.scores.first().copied().unwrap_or(0.0);
    let s1 = outcome.scores.get(1).copied().unwrap_or(0.0);
    db.query(
        "UPDATE $mid SET
             status = 'completed',
             participants[0].score = $s0,
             participants[1].score = $s1,
             completed_at = time::now(),
             updated_at = time::now();",
    )
    .bind(("mid", m.id.clone()))
    .bind(("s0", s0))
    .bind(("s1", s1))
    .await?;
    tracing::info!("Match {} completed: {} {}", m.id.to_sql(), s0, s1);
    Ok(())
}

async fn mark_failed(db: &Database, match_id: &str, err: &str) -> Result<()> {
    db.query(
        "UPDATE type::record('match', $mid) SET
             status = 'failed',
             error_message = $err,
             completed_at = time::now(),
             updated_at = time::now();",
    )
    .bind(("mid", match_id.to_string()))
    .bind(("err", err.to_string()))
    .await?;
    Ok(())
}
