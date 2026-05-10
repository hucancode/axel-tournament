// Match finalizer.
//
// Pure watcher that subscribes to a `LiveRoom` event stream and fires
// the supplied callback the first time a terminal event (`WINNER`,
// `DRAW`, `GAME_END`) is appended. Used by human-vs-human rooms so
// natural game completion gets persisted (match row + room status +
// downstream ELO/finalization) the same way `turn_timer` handles
// timeouts.
//
// The watcher exits after firing the callback once. The room remains
// loaded; downstream finalization decides whether to drop it.

use crate::services::room::bot::parse_terminal;
use crate::services::room::logic::{LiveRoom, RoomLogic};
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

/// Per-player scores and an optional winner index parsed from the
/// terminal event.
#[derive(Debug, Clone)]
pub struct MatchOutcome {
    /// Score per player, ordered to match the room's player list.
    pub scores: Vec<f64>,
    /// Winner index if exactly one player has the strict-max score.
    /// `None` for draws and ties.
    pub winner_idx: Option<usize>,
    /// `kind` of the terminal event (e.g. `WINNER`, `DRAW`, `GAME_END`).
    pub terminal_kind: String,
}

/// Fired once per room when a terminal event lands.
pub type FinishCallback = Arc<
    dyn Fn(String, MatchOutcome) -> futures_util::future::BoxFuture<'static, ()>
        + Send
        + Sync,
>;

pub fn spawn_finish_watcher<L: RoomLogic>(
    room: Arc<LiveRoom<L>>,
    on_finish: FinishCallback,
) -> tokio::task::JoinHandle<()> {
    let mut subscriber = room.subscribe();
    let room_id = room.room_id.clone();
    tokio::spawn(async move {
        loop {
            match subscriber.recv().await {
                Ok(ev) => {
                    let n_players = room
                        .with_state(|s| L::snapshot(s).players.len())
                        .await;
                    if n_players == 0 {
                        continue;
                    }
                    if let Some(scores) = parse_terminal(&ev, n_players) {
                        let outcome = MatchOutcome {
                            winner_idx: pick_winner(&scores),
                            scores,
                            terminal_kind: ev.kind.clone(),
                        };
                        on_finish(room_id.clone(), outcome).await;
                        return;
                    }
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            }
        }
    })
}

fn pick_winner(scores: &[f64]) -> Option<usize> {
    let mut best = f64::NEG_INFINITY;
    let mut best_idx = None;
    let mut tied = false;
    for (i, &s) in scores.iter().enumerate() {
        if s > best {
            best = s;
            best_idx = Some(i);
            tied = false;
        } else if (s - best).abs() < f64::EPSILON {
            tied = true;
        }
    }
    if tied {
        None
    } else {
        best_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_winner_strict_max() {
        assert_eq!(pick_winner(&[1.0, 0.0]), Some(0));
        assert_eq!(pick_winner(&[0.0, 1.0]), Some(1));
        assert_eq!(pick_winner(&[200.0, 100.0]), Some(0));
    }

    #[test]
    fn pick_winner_tie_returns_none() {
        assert_eq!(pick_winner(&[1.0, 1.0]), None);
        assert_eq!(pick_winner(&[0.0, 0.0]), None);
    }
}
