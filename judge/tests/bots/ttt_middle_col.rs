use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut move_count = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();

        if line.starts_with("START") {
            continue;
        } else if line == "YOUR_TURN" {
            let moves = ["MOVE 1 1", "MOVE 2 1"];
            if move_count < moves.len() {
                println!("{}", moves[move_count]);
                stdout.flush().unwrap();
                move_count += 1;
            }
        } else if line.starts_with("SCORE") || line == "END" {
            if line == "END" {
                break;
            }
        }
    }
}
