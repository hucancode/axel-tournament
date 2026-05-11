// Chess starter skeleton. Wire protocol: judge/protocols/wire.md.
// Game spec: judge/protocols/chess.md.
//
// You play `MOVE <from> <to> [promo]` where squares are algebraic
// (`a1`..`h8`). Player 0 = white (moves first), player 1 = black.
// Bots are not told their seat; derive it from the order of MOVE
// events you see before GAME_STARTED finishes.
//
// Replace the body of `choose_move` with your engine. Returning
// `None` skips a turn — useful while you scaffold.

use std::io::{self, BufRead, Write};

#[derive(Default)]
struct State {
    // TODO: track board, side-to-move, my_seat, last_move, etc.
    move_count: u32,
}

fn choose_move(_state: &State) -> Option<String> {
    // TODO: return e.g. Some("e2 e4 -".into()).
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
        let payload: Vec<&str> = tok.collect();

        match kind {
            "GAME_STARTED" => {
                if let Some(act) = choose_move(&state) {
                    let _ = writeln!(stdout, "ACT MOVE {act}");
                    let _ = stdout.flush();
                }
            }
            "MOVE" => {
                state.move_count += 1;
                // payload = [pid, from, to, flag]
                let _ = payload;
                if let Some(act) = choose_move(&state) {
                    let _ = writeln!(stdout, "ACT MOVE {act}");
                    let _ = stdout.flush();
                }
            }
            "WINNER" | "DRAW" => break,
            _ => {}
        }
    }
}
