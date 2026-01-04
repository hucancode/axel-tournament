use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut move_count = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();

        if line.starts_with("START") {
            // Game started, we know our symbol
            continue;
        } else if line == "YOUR_TURN" {
            // Our turn to move - play top row strategy
            let moves = ["MOVE 0 0", "MOVE 0 1", "MOVE 0 2"];
            if move_count < moves.len() {
                println!("{}", moves[move_count]);
                stdout.flush().unwrap();
                move_count += 1;
            }
        } else if line.starts_with("SCORE") || line == "END" {
            // Game over
            if line == "END" {
                break;
            }
        }
        // Ignore board states (multi-line messages)
    }
}
