// Sample bot for jar-of-greed: contributes a fixed fraction (one third)
// of its current coin stack each round, rounded down. Tracks own coin
// balance from its own contributions + the per-round payout. Per-player
// balances are not on the wire during play, so the bot can only see
// other players' balances at GAME_END (the final reveal).

use crate::services::playground::PlaygroundStrategy;

pub struct JogStrategy {
    coins: i64,
    last_contribution: i64,
}

impl Default for JogStrategy {
    fn default() -> Self {
        Self {
            coins: 0,
            last_contribution: 0,
        }
    }
}

impl PlaygroundStrategy for JogStrategy {
    fn react(&mut self, _bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "GAME_STARTED" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                let starting = parts.first().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                self.coins = starting;
                let amt = self.coins / 3;
                self.last_contribution = amt;
                self.coins -= amt;
                Some(("CONTRIBUTE".into(), amt.to_string()))
            }
            "ROUND_RESULT" => {
                // payload: "<round> <multiplier> <jar> <payout>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                let payout = parts.get(3).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                self.coins += payout;
                if self.coins <= 0 {
                    self.last_contribution = 0;
                    return Some(("CONTRIBUTE".into(), "0".into()));
                }
                let amt = self.coins / 3;
                self.last_contribution = amt;
                self.coins -= amt;
                Some(("CONTRIBUTE".into(), amt.to_string()))
            }
            _ => None,
        }
    }
}
