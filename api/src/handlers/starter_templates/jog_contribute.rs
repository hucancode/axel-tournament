// Reference Jar-of-Greed bot. Contributes a fixed amount each round.
//
// Wire protocol: judge/protocols/wire.md (stdio transport).
// Game spec: judge/protocols/jar-of-greed.md.
//
// Strategy: on GAME_STARTED contribute, then re-contribute each time
// a ROUND_RESULT signals the next round has begun. The room rejects
// duplicate CONTRIBUTEs in the same round, so one ACT per round is
// the safe pattern.

use std::io::{self, BufRead, Write};

const CONTRIBUTION: u32 = 1;

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
                let _ = writeln!(stdout, "ACT CONTRIBUTE {CONTRIBUTION}");
                let _ = stdout.flush();
            }
            Some("GAME_END") => break,
            _ => {}
        }
    }
}
