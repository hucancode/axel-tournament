// Reference RPS bot. Always plays PAPER. Spec: judge/protocols/wire.md.

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
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
            Some("GAME_STARTED") | Some("ROUND_RESULT") => {
                let _ = writeln!(stdout, "ACT MOVE PAPER");
                let _ = stdout.flush();
            }
            Some("GAME_END") => break,
            _ => {}
        }
    }
}
