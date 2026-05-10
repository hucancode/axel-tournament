// Reference RPS bot. Always plays PAPER. Spec: judge/protocols/wire.md.
// Tracks total rounds from GAME_STARTED so the post-final move is
// suppressed (phase=Finished would reject it).

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut total_rounds: u32 = 0;
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
            Some("GAME_STARTED") => {
                total_rounds = tok.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                let _ = writeln!(stdout, "ACT MOVE PAPER");
                let _ = stdout.flush();
            }
            Some("ROUND_RESULT") => {
                let round: u32 = tok.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                if round < total_rounds {
                    let _ = writeln!(stdout, "ACT MOVE PAPER");
                    let _ = stdout.flush();
                }
            }
            Some("GAME_END") => break,
            _ => {}
        }
    }
}
