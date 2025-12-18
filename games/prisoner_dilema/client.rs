use std::io::{self, BufRead};

fn main() {
    // Simple prisoner's dilemma strategy: Tit-for-Tat
    // Start with cooperation, then mirror opponent's last move
    // This is a classic and effective strategy for iterated prisoner's dilemma
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut opponent_last_move: Option<String> = None;
    while let Some(Ok(line)) = lines.next() {
        let command = line.trim();
        match command {
            "MOVE" => {
                if let Some(ref opp_move) = opponent_last_move {
                    println!("{}", opp_move);
                } else {
                    println!("cooperate");
                }
            }
            cmd if cmd.starts_with("OPP ") => {
                let opp_move = cmd.strip_prefix("OPP ").unwrap_or("").trim();
                opponent_last_move = Some(opp_move.to_string());
            }
            "END" => {
                break;
            }
            _ => {
                eprintln!("Unknown command: {}", command);
            }
        }
    }
}
