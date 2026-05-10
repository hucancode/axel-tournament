// Deterministic RPS playground bot: cycles ROCK, PAPER, SCISSORS, ROCK, ...
// Determinism matters for replay; a time-based picker drifts across
// runs and makes the playground non-reproducible.

use crate::services::playground::PlaygroundStrategy;

#[derive(Default)]
pub struct RpsStrategy {
    move_index: usize,
}

impl PlaygroundStrategy for RpsStrategy {
    fn react(&mut self, _bot_pid: &str, kind: &str, _payload: &str) -> Option<(String, String)> {
        match kind {
            "GAME_STARTED" | "ROUND_RESULT" => {
                let m = match self.move_index % 3 {
                    0 => "ROCK",
                    1 => "PAPER",
                    _ => "SCISSORS",
                };
                self.move_index += 1;
                Some(("MOVE".into(), m.into()))
            }
            _ => None,
        }
    }
}
