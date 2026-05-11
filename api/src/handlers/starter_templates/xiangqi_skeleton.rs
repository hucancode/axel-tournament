// Xiangqi (Chinese chess) starter skeleton. Wire protocol:
// judge/protocols/wire.md. Game spec: judge/protocols/xiangqi.md.
//
// Coordinates: `<file><rank>` where file is `a..i` and rank is `0..9`.
// Player 0 = red, moves first. Player 1 = black. Replace
// `choose_move` with your engine; return `None` to skip the turn.

use std::io::{self, BufRead, Write};

#[derive(Default)]
struct State {
    // TODO: track board, side-to-move, my_seat, last_move, ...
    move_count: u32,
}

fn choose_move(_state: &State) -> Option<String> {
    // TODO: return e.g. Some("b2 e2".into()) to advance the red cannon.
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
            "GAME_STARTED" => {
                if let Some(mv) = choose_move(&state) {
                    let _ = writeln!(stdout, "ACT MOVE {mv}");
                    let _ = stdout.flush();
                }
            }
            "MOVE" => {
                state.move_count += 1;
                if let Some(mv) = choose_move(&state) {
                    let _ = writeln!(stdout, "ACT MOVE {mv}");
                    let _ = stdout.flush();
                }
            }
            "WINNER" => break,
            _ => {}
        }
    }
}
