// Bot-vs-bot matches end-to-end through the production code path:
// real source files compiled by `CompilerSandbox`, spawned as
// sandboxed subprocesses, then driven by
// `services::room::bot::run_match` against a `LiveRoom<Rps>`.
//
// Requires Linux + cgroups v2 + the privileges the sandbox needs
// (same as `tests/pentest.rs`). DB-free: the room is built on
// `Storage::memory()`.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use judge::games::Rps;
use judge::services::room::bot::{run_match, wrap, MatchOutcome};
use judge::services::room::logic::{LiveRoom, RoomRegistry};
use judge::services::sandbox::{BuildSandbox as CompilerSandbox, SandboxedBot};
use judge::services::storage::Storage;
use tempfile::TempDir;

const LEASE: Duration = Duration::from_secs(30);
const TURN_TIMEOUT: Duration = Duration::from_secs(5);
const RPS_DEFAULT_ROUNDS: usize = 5;

fn unique(p: &str) -> String {
    format!("{p}_{}", uuid::Uuid::new_v4().simple())
}

/// Compile a bot source file from `judge/tests/bots/<name>.rs`.
/// Returns the absolute path to the produced binary plus the temp dir
/// that owns the workspace (must outlive the binary).
async fn compile_bot(name: &str) -> (TempDir, PathBuf) {
    let src_path = format!("tests/bots/{name}.rs");
    let source = fs::read_to_string(&src_path)
        .unwrap_or_else(|e| panic!("read bot source {src_path}: {e}"));
    let temp_dir = TempDir::new().unwrap();
    let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();
    let bin = sandbox
        .compile(&unique(name), "rust", &source)
        .await
        .unwrap_or_else(|e| panic!("compile {name}: {e}"));
    (temp_dir, PathBuf::from(bin))
}

async fn fresh_room() -> (Arc<RoomRegistry<Rps>>, Arc<LiveRoom<Rps>>, String) {
    let storage = Storage::memory();
    let registry = Arc::new(RoomRegistry::<Rps>::new(storage, unique("judge")));
    let room_id = unique("room");
    let live = registry.open(&room_id, LEASE).await.unwrap();
    (registry, live, room_id)
}

/// Compile two bots, spawn them sandboxed, run a full Rps match.
async fn play(
    bot_a: &str,
    bot_b: &str,
) -> (MatchOutcome, Arc<LiveRoom<Rps>>, Arc<RoomRegistry<Rps>>, String) {
    let (_t_a, bin_a) = compile_bot(bot_a).await;
    let (_t_b, bin_b) = compile_bot(bot_b).await;

    let pid_a = unique(bot_a);
    let pid_b = unique(bot_b);

    let conn_a = wrap(SandboxedBot::spawn(&pid_a, &bin_a).await.expect("spawn A"));
    let conn_b = wrap(SandboxedBot::spawn(&pid_b, &bin_b).await.expect("spawn B"));

    let (registry, room, room_id) = fresh_room().await;
    let outcome = run_match(
        room.clone(),
        vec![conn_a, conn_b],
        vec![pid_a, pid_b],
        TURN_TIMEOUT,
    )
    .await
    .expect("run_match");

    // Hold the temp dirs alive past run_match by leaking the bindings
    // out of scope: returning the outcome is enough since the binaries
    // were already exec'd into the children's memory by the kernel.
    (outcome, room, registry, room_id)
}

// ---------- Tests ----------

/// Always-rock vs always-rock: every round is a tie.
#[tokio::test]
async fn rock_vs_rock_all_draws() {
    let (outcome, _room, registry, room_id) = play("rps_rock", "rps_rock").await;
    assert!(outcome.faulted_indices.is_empty(), "fault: {:?}", outcome.fault_reason);
    assert_eq!(outcome.scores, vec![0.0, 0.0]);
    registry.drop_room(&room_id).await;
}

/// Always-rock vs always-paper: paper sweeps 0-5.
#[tokio::test]
async fn rock_vs_paper_paper_sweeps() {
    let (outcome, _room, registry, room_id) = play("rps_rock", "rps_paper").await;
    assert!(outcome.faulted_indices.is_empty(), "fault: {:?}", outcome.fault_reason);
    assert_eq!(outcome.scores, vec![0.0, RPS_DEFAULT_ROUNDS as f64]);
    registry.drop_room(&room_id).await;
}

/// Cycle (ROCK/PAPER/SCISSORS/ROCK/PAPER) vs always-rock over 5 rounds.
/// Per round: draw, cycle wins, rock wins, draw, cycle wins -> 2-1.
#[tokio::test]
async fn cycle_vs_rock_deterministic_score() {
    let (outcome, _room, registry, room_id) = play("rps_cycle", "rps_rock").await;
    assert!(outcome.faulted_indices.is_empty(), "fault: {:?}", outcome.fault_reason);
    assert_eq!(outcome.scores, vec![2.0, 1.0]);
    registry.drop_room(&room_id).await;
}

/// Verify the event log was the full match: 5 ROUND_RESULTs + GAME_END.
#[tokio::test]
async fn cycle_vs_rock_event_log_shape() {
    let (outcome, room, registry, room_id) = play("rps_cycle", "rps_rock").await;
    assert!(outcome.faulted_indices.is_empty(), "fault: {:?}", outcome.fault_reason);

    let events = room.read_since(0).await.unwrap();
    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();

    assert_eq!(&kinds[..3], &["PLAYER_JOINED", "PLAYER_JOINED", "GAME_STARTED"]);
    assert_eq!(events.last().unwrap().kind, "GAME_END");
    assert_eq!(
        events.iter().filter(|e| e.kind == "ROUND_RESULT").count(),
        RPS_DEFAULT_ROUNDS
    );
    for (i, ev) in events.iter().enumerate() {
        assert_eq!(ev.seq, (i + 1) as u64, "seq must be monotonic from 1");
    }
    registry.drop_room(&room_id).await;
}
