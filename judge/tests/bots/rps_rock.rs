// Reference RPS bot. Always plays ROCK.
//
// Wire protocol: judge/protocols/wire.md (stdio transport).
// Reads `EVENT seq kind payload` from stdin, writes `ACT kind payload`
// to stdout. The orchestrator JOINs and STARTs the bot; this binary
// only needs to react to GAME_STARTED + ROUND_RESULT with a move.

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
                let _ = writeln!(stdout, "ACT MOVE ROCK");
                let _ = stdout.flush();
            }
            Some("GAME_END") => break,
            _ => {}
        }
    }
}
