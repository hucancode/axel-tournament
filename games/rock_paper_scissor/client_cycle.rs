use std::io::{self, BufRead};

fn main() {
    // Simple rock-paper-scissor strategy
    // This is a naive implementation that alternates between moves
    // Real players can implement more sophisticated strategies
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let moves = ["rock", "paper", "scissor"];
    let mut move_index = 0;
    let mut opponent_last_move: Option<String> = None;
    while let Some(Ok(line)) = lines.next() {
        let command = line.trim();
        match command {
            "MOVE" => {
                println!("{}", moves[move_index]);
                move_index = (move_index + 1) % 3;
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
