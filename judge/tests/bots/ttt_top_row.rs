// Reference TTT bot. Tries the top row left-to-right.
//
// Wire protocol: judge/protocols/wire.md (stdio transport).
//
// Bots don't know their seat (X or O). Strategy: react to GAME_STARTED
// and every MOVE event by attempting the next cell in the bot's
// preferred sequence. The server silently ignores ACTs that aren't the
// bot's turn or hit an occupied cell, so the off-turn attempts are
// harmless. The bot keeps advancing through its list whenever any
// MOVE event lands.

use std::io::{self, BufRead, Write};

fn main() {
    let cells: [(u8, u8); 9] = [
        (0, 0), (0, 1), (0, 2),
        (1, 0), (1, 1), (1, 2),
        (2, 0), (2, 1), (2, 2),
    ];
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut idx = 0usize;

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
        match tok.next() {
            Some("GAME_STARTED") | Some("MOVE") => {
                if idx < cells.len() {
                    let (r, c) = cells[idx];
                    let _ = writeln!(stdout, "ACT MOVE {r} {c}");
                    let _ = stdout.flush();
                    idx += 1;
                }
            }
            Some("WINNER") | Some("DRAW") => break,
            _ => {}
        }
    }
}
