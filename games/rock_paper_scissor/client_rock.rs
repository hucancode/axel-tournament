use std::io::{self, BufRead};

fn main() {
    // Always plays rock regardless of previous moves.
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        let input = line.trim();
        match input {
            "MOVE" => println!("rock"),
            "END" => break,
            // Ignore opponent move updates or unexpected commands
            _ => {}
        }
    }
}
