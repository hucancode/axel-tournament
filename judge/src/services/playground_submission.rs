// Playground variant: the human plays against their own compiled bot
// (subprocess transport, same wire protocol as bot-vs-bot matches),
// not the in-process sample strategy. Used by
// `POST /api/playground/start_with_submission`.

use crate::services::room::bot::{Bot, wrap};
use crate::services::room::logic::{LiveRoom, RoomLogic, RoomRegistry};
use crate::services::sandbox::SandboxedBot;
use crate::services::storage::Event;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::broadcast::error::RecvError;

const LEASE_TTL: Duration = Duration::from_secs(30);
const BOT_PID_PREFIX: &str = "bot:submission:";

#[async_trait]
pub trait SubmissionPlaygroundHost: Send + Sync + 'static {
    /// Open a fresh playground room, spawn the user's bot, and return
    /// `(room_id, bot_player_id)`. The driver bridges the bot to the
    /// room in the background; the room is dropped when the bot exits
    /// or the match terminates.
    async fn spawn(
        &self,
        submission_id: String,
        binary_path: PathBuf,
        turn_timeout: Duration,
    ) -> Result<(String, String)>;
}

pub type SubmissionPlaygroundRegistries =
    HashMap<&'static str, Arc<dyn SubmissionPlaygroundHost>>;

pub fn host<L: RoomLogic>(
    registry: Arc<RoomRegistry<L>>,
) -> Arc<dyn SubmissionPlaygroundHost> {
    Arc::new(GameHost { registry })
}

struct GameHost<L: RoomLogic> {
    registry: Arc<RoomRegistry<L>>,
}

#[async_trait]
impl<L: RoomLogic> SubmissionPlaygroundHost for GameHost<L> {
    async fn spawn(
        &self,
        submission_id: String,
        binary_path: PathBuf,
        turn_timeout: Duration,
    ) -> Result<(String, String)> {
        let room_id = format!("playground-{}", uuid::Uuid::new_v4());
        let bot_pid = format!("{BOT_PID_PREFIX}{submission_id}");

        let room = self.registry.open(&room_id, LEASE_TTL).await?;
        let sandboxed = SandboxedBot::spawn(&bot_pid, &binary_path)
            .await
            .map_err(|e| anyhow!("spawn sandboxed bot: {e}"))?;
        let bot = wrap(sandboxed);

        let registry = self.registry.clone();
        let room_id_for_driver = room_id.clone();
        let bot_pid_for_driver = bot_pid.clone();
        tokio::spawn(async move {
            if let Err(e) = drive(room, bot, bot_pid_for_driver, turn_timeout).await {
                tracing::debug!("submission playground driver exited: {e}");
            }
            registry.drop_room(&room_id_for_driver).await;
        });

        Ok((room_id, bot_pid))
    }
}

async fn drive<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    bot: Bot,
    bot_pid: String,
    turn_timeout: Duration,
) -> Result<()> {
    let mut sub = room.subscribe();

    let backlog = room.read_since(0).await?;
    let mut joined = false;
    for ev in &backlog {
        if !joined
            && ev.kind == "PLAYER_JOINED"
            && ev.payload.trim() != bot_pid
        {
            room.handle_act(&bot_pid, "JOIN", "").await.ok();
            joined = true;
        }
        send_event(&bot, ev).await.ok();
    }

    // Reader: pull ACT frames from bot stdout, dispatch to room.
    let reader_bot = bot.clone();
    let reader_room = room.clone();
    let reader_pid = bot_pid.clone();
    let reader = tokio::spawn(async move {
        loop {
            let line = match recv_line(&reader_bot, turn_timeout).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::debug!("submission bot read end: {e}");
                    return;
                }
            };
            if line.is_empty() {
                continue;
            }
            // ACT kind [payload]
            let mut tok = line.splitn(3, ' ');
            if tok.next() != Some("ACT") {
                continue;
            }
            let Some(kind) = tok.next() else { continue };
            let payload = tok.next().unwrap_or("");
            if let Err(e) = reader_room.handle_act(&reader_pid, kind, payload).await {
                tracing::debug!("submission bot ACT rejected: {e}");
            }
        }
    });

    loop {
        match sub.recv().await {
            Ok(ev) => {
                if !joined
                    && ev.kind == "PLAYER_JOINED"
                    && ev.payload.trim() != bot_pid
                {
                    room.handle_act(&bot_pid, "JOIN", "").await.ok();
                    joined = true;
                }
                if send_event(&bot, &ev).await.is_err() {
                    break;
                }
                if ev.kind == "GAME_END"
                    || ev.kind == "WINNER"
                    || ev.kind == "DRAW"
                {
                    break;
                }
            }
            Err(RecvError::Lagged(n)) => {
                tracing::warn!("submission playground subscriber lagged by {n}");
            }
            Err(RecvError::Closed) => break,
        }
    }

    reader.abort();
    Ok(())
}

async fn send_event(bot: &Bot, e: &Event) -> Result<()> {
    let line = if e.payload.is_empty() {
        format!("EVENT {} {}", e.seq, e.kind)
    } else {
        format!("EVENT {} {} {}", e.seq, e.kind, e.payload)
    };
    let mut g = bot.lock().await;
    g.stdin.write_all(line.as_bytes()).await?;
    g.stdin.write_all(b"\n").await?;
    g.stdin.flush().await?;
    Ok(())
}

async fn recv_line(bot: &Bot, timeout: Duration) -> Result<String> {
    let mut g = bot.lock().await;
    let mut line = String::new();
    let n = tokio::time::timeout(timeout, g.stdout.read_line(&mut line))
        .await
        .map_err(|_| anyhow!("bot read timeout"))??;
    if n == 0 {
        return Err(anyhow!("bot eof"));
    }
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}
