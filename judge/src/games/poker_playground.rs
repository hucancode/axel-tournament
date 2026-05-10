// Deterministic Poker playground bot.
//
// Strategy: always CHECK if it's free, otherwise CALL if the price is
// cheap (≤ 50 chips), otherwise FOLD. Never bets, never raises, never
// goes all-in. The bot tracks just enough state from the event log to
// know when it is to act and what it owes to call.

use crate::services::playground::PlaygroundStrategy;

const CHEAP_CALL_LIMIT: i64 = 50;

#[derive(Default)]
pub struct PokerStrategy {
    players: Vec<String>,
    bot_idx: Option<usize>,
    dealer: u8,
    /// `to_act` tracks the current actor (0 or 1) on the active street.
    to_act: u8,
    bet_in_street: [i64; 2],
    in_hand: bool,
    /// Last raise size on this street, used to detect min-raise gates
    /// (not needed by the strategy but kept for parity with logic).
    last_raise_size: i64,
}

impl PokerStrategy {
    fn idx_of(&self, pid: &str) -> Option<usize> {
        self.players.iter().position(|p| p == pid)
    }

    fn am_i_to_act(&self) -> bool {
        match self.bot_idx {
            Some(i) => i as u8 == self.to_act,
            None => false,
        }
    }

    fn to_call(&self) -> i64 {
        let me = self.bot_idx.unwrap_or(0);
        let opp = 1 - me;
        (self.bet_in_street[opp] - self.bet_in_street[me]).max(0)
    }

    fn pick_action(&self) -> (String, String) {
        let to_call = self.to_call();
        if to_call == 0 {
            return ("CHECK".into(), String::new());
        }
        if to_call <= CHEAP_CALL_LIMIT {
            return ("CALL".into(), String::new());
        }
        ("FOLD".into(), String::new())
    }

    fn apply_action(&mut self, pid: &str, verb: &str, amt: i64) {
        let Some(i) = self.idx_of(pid) else {
            return;
        };
        match verb {
            "POST" => {
                self.bet_in_street[i] = amt;
                if amt > self.last_raise_size {
                    self.last_raise_size = amt;
                }
                self.to_act = self.dealer; // dealer acts first preflop
            }
            "FOLD" => {
                self.in_hand = false;
            }
            "CHECK" => {
                self.to_act = 1 - i as u8;
            }
            "CALL" => {
                self.bet_in_street[i] += amt;
                self.to_act = 1 - i as u8;
            }
            "BET" | "RAISE" => {
                // Use total of the actor's contribution this street.
                self.bet_in_street[i] = self.bet_in_street[i].max(amt);
                self.last_raise_size = self.bet_in_street[i];
                self.to_act = 1 - i as u8;
            }
            "ALLIN" => {
                self.bet_in_street[i] += amt;
                self.to_act = 1 - i as u8;
            }
            _ => {}
        }
    }
}

impl PlaygroundStrategy for PokerStrategy {
    fn react(&mut self, bot_pid: &str, kind: &str, payload: &str) -> Option<(String, String)> {
        match kind {
            "PLAYER_JOINED" => {
                let pid = payload.trim().to_string();
                if !self.players.contains(&pid) {
                    self.players.push(pid);
                }
                self.bot_idx = self.idx_of(bot_pid);
                None
            }
            "GAME_STARTED" => {
                self.bot_idx = self.idx_of(bot_pid);
                None
            }
            "HAND_STARTED" => {
                // payload: "<hand_no> <dealer_idx> <hole0a> <hole0b> <hole1a> <hole1b>"
                let parts: Vec<&str> = payload.split_whitespace().collect();
                self.dealer = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                self.bet_in_street = [0, 0];
                self.last_raise_size = 0;
                self.in_hand = true;
                self.to_act = self.dealer;
                None
            }
            "STREET" => {
                self.bet_in_street = [0, 0];
                self.last_raise_size = 0;
                // Postflop opponent of dealer acts first.
                self.to_act = 1 - self.dealer;
                if self.am_i_to_act() {
                    Some(self.pick_action())
                } else {
                    None
                }
            }
            "ACTION" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() < 2 {
                    return None;
                }
                let pid = parts[0];
                let verb = parts[1];
                let amt: i64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                self.apply_action(pid, verb, amt);
                if self.in_hand && self.am_i_to_act() {
                    Some(self.pick_action())
                } else {
                    None
                }
            }
            "HAND_END" => {
                self.in_hand = false;
                self.bet_in_street = [0, 0];
                None
            }
            _ => None,
        }
    }
}
