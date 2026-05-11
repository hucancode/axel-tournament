// Heads-up no-limit Texas Hold'em starter skeleton.
// Wire protocol: judge/protocols/wire.md. Game spec:
// judge/protocols/poker.md.
//
// Each event carries one of: HAND_STARTED, STREET, ACTION, POT,
// HAND_END, GAME_END, WINNER, DRAW. You act with FOLD / CHECK / CALL
// / BET <amt> / RAISE <to-amt> / ALLIN. Implement `decide` to return
// the next action when it is your turn.

use std::io::{self, BufRead, Write};

#[derive(Default)]
struct State {
    // TODO: hand_no, dealer_idx, my_seat, hole, board, pot, last_action, ...
}

fn decide(_state: &State) -> Option<String> {
    // TODO: return e.g. Some("FOLD".into()) or Some("BET 50".into()).
    None
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut state = State::default();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let mut tok = line.split_whitespace();
        if tok.next() != Some("EVENT") {
            continue;
        }
        let _seq = tok.next();
        let kind = match tok.next() {
            Some(k) => k,
            None => continue,
        };

        match kind {
            "HAND_STARTED" | "STREET" | "ACTION" | "POT" => {
                if let Some(act) = decide(&state) {
                    let _ = writeln!(stdout, "ACT {act}");
                    let _ = stdout.flush();
                }
            }
            "GAME_END" | "WINNER" | "DRAW" => break,
            _ => {}
        }
    }
    let _ = &mut state;
}
