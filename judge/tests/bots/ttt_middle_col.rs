// Reference TTT bot. Tries the middle column top-to-bottom, then
// fills remaining cells in row-major order.
//
// Same idea as ttt_top_row.rs: react to GAME_STARTED + every MOVE
// event by attempting the next cell; off-turn / occupied cells are
// silently ignored by the server, so the bot keeps advancing.

use std::io::{self, BufRead, Write};

fn main() {
    let cells: [(u8, u8); 9] = [
        (0, 1), (1, 1), (2, 1),
        (0, 0), (1, 0), (2, 0),
        (0, 2), (1, 2), (2, 2),
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
