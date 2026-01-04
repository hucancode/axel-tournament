use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();

        if line == "START" {
            // Game started, send first move
            println!("PAPER");
            stdout.flush().unwrap();
        } else if line.starts_with("ROUND") {
            // Round result received, send next move
            println!("PAPER");
            stdout.flush().unwrap();
        } else if line.starts_with("SCORE") {
            // Final score received
            continue;
        } else if line == "END" {
            // Game over
            break;
        }
    }
}
