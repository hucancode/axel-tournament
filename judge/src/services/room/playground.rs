// In-process playground bot. Used by the protocol-learning sandbox so a
// participant can connect to a real room and play against a deterministic
// sample bot. Unlike `BotConn`, this driver does not spawn a sandbox
// subprocess: it folds events from `LiveRoom::subscribe` and replays a
// strategy by calling `LiveRoom::handle_act` directly.
//
// The bot is server-authored, so it has no JWT and no transport — it
// simply uses a synthetic `player_id` (e.g. `bot:sample`) which
// `RoomLogic::validate` accepts the same as any other id.

use crate::games::{Pd, Rps, Ttt};
use crate::services::room::logic::{LiveRoom, RoomLogic, RoomRegistry};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;

const PLAYGROUND_LEASE_TTL: Duration = Duration::from_secs(15);
const BOT_PLAYER_ID: &str = "bot:sample";

/// Strategy interface: react to one event and (optionally) emit one ACT.
trait BotStrategy: Send + 'static {
    /// Called once when the bot starts, after JOIN.
    fn on_attach(&mut self, _bot_pid: &str) {}

    /// React to a freshly committed event. Return Some((kind,payload)) to
    /// send an ACT, or None to wait for the next event.
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)>;
}

/// Spawn a playground bot for an open RPS room. The bot JOINs immediately
/// (so the human becomes host on their own JOIN — the human is the first
/// to JOIN since rooms are dedicated). Actually the ordering here is the
/// reverse: the bot is created BEFORE the human connects, so the bot is
/// host and starts the match. Either way is teaching-equivalent — we let
/// the human start so they see GAME_STARTED happen on a wire frame they
/// triggered. So: bot waits for human JOIN, then JOINs second.
pub async fn spawn_rps(
    registry: Arc<RoomRegistry<Rps>>,
    room_id: String,
) -> Result<String> {
    let room = registry.open(&room_id, PLAYGROUND_LEASE_TTL).await?;
    spawn::<Rps, _>(registry, room, room_id, RpsStrategy::default());
    Ok(BOT_PLAYER_ID.to_string())
}

pub async fn spawn_ttt(
    registry: Arc<RoomRegistry<Ttt>>,
    room_id: String,
) -> Result<String> {
    let room = registry.open(&room_id, PLAYGROUND_LEASE_TTL).await?;
    spawn::<Ttt, _>(registry, room, room_id, TttStrategy::default());
    Ok(BOT_PLAYER_ID.to_string())
}

pub async fn spawn_pd(
    registry: Arc<RoomRegistry<Pd>>,
    room_id: String,
) -> Result<String> {
    let room = registry.open(&room_id, PLAYGROUND_LEASE_TTL).await?;
    spawn::<Pd, _>(registry, room, room_id, PdStrategy::default());
    Ok(BOT_PLAYER_ID.to_string())
}

fn spawn<L, S>(
    registry: Arc<RoomRegistry<L>>,
    room: Arc<LiveRoom<L>>,
    room_id: String,
    mut strategy: S,
)
where
    L: RoomLogic,
    S: BotStrategy,
{
    tokio::spawn(async move {
        if let Err(e) = drive(room, &mut strategy).await {
            tracing::debug!("playground bot exited: {e}");
        }
        // Release the lease + drop from registry so the playground room
        // doesn't accumulate forever.
        registry.drop_room(&room_id).await;
    });
}

async fn drive<L, S>(room: Arc<LiveRoom<L>>, strategy: &mut S) -> Result<()>
where
    L: RoomLogic,
    S: BotStrategy,
{
    let bot_pid = BOT_PLAYER_ID.to_string();
    strategy.on_attach(&bot_pid);

    let mut sub = room.subscribe();

    // Replay backlog so the bot reacts to events emitted before it
    // subscribed (e.g. an early human JOIN).
    let backlog = room.read_since(0).await?;
    let mut joined = false;
    for e in &backlog {
        if !joined && e.kind == "PLAYER_JOINED" && e.payload.trim() != bot_pid {
            // Human already in the room — bot joins now as the second player.
            let _ = room.handle_act(&bot_pid, "JOIN", "").await;
            joined = true;
        }
        if let Some((kind, payload)) = strategy.react(&bot_pid, &e.kind, &e.payload) {
            let _ = room.handle_act(&bot_pid, &kind, &payload).await;
        }
    }

    loop {
        match sub.recv().await {
            Ok(event) => {
                if !joined && event.kind == "PLAYER_JOINED" && event.payload.trim() != bot_pid {
                    let _ = room.handle_act(&bot_pid, "JOIN", "").await;
                    joined = true;
                }
                if event.kind == "GAME_END"
                    || event.kind == "WINNER"
                    || event.kind == "DRAW"
                {
                    return Ok(());
                }
                if let Some((kind, payload)) = strategy.react(&bot_pid, &event.kind, &event.payload)
                {
                    let _ = room.handle_act(&bot_pid, &kind, &payload).await;
                }
            }
            Err(RecvError::Lagged(n)) => {
                tracing::warn!("playground bot lagged by {n}; continuing");
            }
            Err(RecvError::Closed) => return Err(anyhow!("event stream closed")),
        }
    }
}

// --- strategies ---

#[derive(Default)]
struct RpsStrategy;

impl BotStrategy for RpsStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "GAME_STARTED" => Some(("MOVE".into(), pick_rps())),
            "ROUND_RESULT" => {
                // payload: "<round> <m0> <m1> <s0> <s1>" — match not over here;
                // GAME_END is its own event. Submit next move.
                let _ = (bot_pid, payload);
                Some(("MOVE".into(), pick_rps()))
            }
            _ => None,
        }
    }
}

fn pick_rps() -> String {
    // Cheap deterministic-ish source. Don't pull in `rand` — every
    // strategy reaches for it and the pattern matters less than the
    // protocol shape we're teaching.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    match nanos % 3 {
        0 => "ROCK".into(),
        1 => "PAPER".into(),
        _ => "SCISSORS".into(),
    }
}

#[derive(Default)]
struct TttStrategy {
    board: [Option<char>; 9], // 'X' / 'O'
    bot_mark: Option<char>,
    players: Vec<String>,
}

impl BotStrategy for TttStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !self.players.contains(&pid) {
                    self.players.push(pid);
                }
                None
            }
            "GAME_STARTED" => {
                // Player 0 = X, plays first.
                self.bot_mark = self
                    .players
                    .iter()
                    .position(|p| p == bot_pid)
                    .map(|i| if i == 0 { 'X' } else { 'O' });
                if self.bot_mark == Some('X') {
                    self.first_empty_move()
                } else {
                    None
                }
            }
            "MOVE" => {
                // payload: "<pid> <row> <col>"
                let mut it = payload.split_whitespace();
                let pid = it.next()?;
                let row: usize = it.next()?.parse().ok()?;
                let col: usize = it.next()?.parse().ok()?;
                let idx = row * 3 + col;
                let mark = self
                    .players
                    .iter()
                    .position(|p| p == pid)
                    .map(|i| if i == 0 { 'X' } else { 'O' })?;
                if idx < 9 {
                    self.board[idx] = Some(mark);
                }
                if pid != bot_pid {
                    self.first_empty_move()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl TttStrategy {
    fn first_empty_move(&self) -> Option<(String, String)> {
        let idx = self.board.iter().position(|c| c.is_none())?;
        let row = idx / 3;
        let col = idx % 3;
        Some(("MOVE".into(), format!("{row} {col}")))
    }
}

/// Tit-for-tat: cooperate first, then mirror the opponent's last move.
#[derive(Default)]
struct PdStrategy {
    last_opponent: Option<char>,
}

impl BotStrategy for PdStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "GAME_STARTED" => {
                self.last_opponent = None;
                Some(("MOVE".into(), "C".into()))
            }
            "ROUND_RESULT" => {
                // payload: "<round> <m0> <m1> <s0> <s1>"
                // Opponent's move is whichever of m0/m1 is not the bot's.
                // We don't know our index here cheaply, so use the bot's
                // own MOVE event tracking instead. Just default to last
                // opponent if known.
                let _ = payload;
                let next = match self.last_opponent {
                    Some('D') => "D",
                    _ => "C",
                };
                Some(("MOVE".into(), next.into()))
            }
            "MOVE" => {
                // payload: "<pid> <C|D>"
                let mut it = payload.split_whitespace();
                let pid = it.next()?;
                let m = it.next()?.chars().next()?;
                if pid != bot_pid {
                    self.last_opponent = Some(m);
                }
                None
            }
            _ => None,
        }
    }
}
