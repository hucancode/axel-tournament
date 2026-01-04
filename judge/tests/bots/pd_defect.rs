use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();

        if line == "START" {
            println!("DEFECT");
            stdout.flush().unwrap();
        } else if line.starts_with("RESULT") {
            println!("DEFECT");
            stdout.flush().unwrap();
        } else if line.starts_with("SCORE") {
            continue;
        } else if line == "END" {
            break;
        }
    }
}
