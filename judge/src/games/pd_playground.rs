// Tit-for-tat: cooperate first, then mirror the opponent's last move.

use crate::services::playground::PlaygroundStrategy;

#[derive(Default)]
pub struct PdStrategy {
    last_opponent: Option<char>,
}

impl PlaygroundStrategy for PdStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "GAME_STARTED" => {
                self.last_opponent = None;
                Some(("MOVE".into(), "C".into()))
            }
            "ROUND_RESULT" => {
                let next = match self.last_opponent {
                    Some('D') => "D",
                    _ => "C",
                };
                Some(("MOVE".into(), next.into()))
            }
            "MOVE" => {
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
