use rand::Rng;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Move {
    Cooperate,
    Defect,
}

impl Move {
    fn from_str(s: &str) -> Option<Move> {
        match s.to_lowercase().trim() {
            "cooperate" | "c" => Some(Move::Cooperate),
            "defect" | "d" => Some(Move::Defect),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Move::Cooperate => "cooperate",
            Move::Defect => "defect",
        }
    }
}

fn calculate_scores(move1: Move, move2: Move) -> (i32, i32) {
    match (move1, move2) {
        (Move::Cooperate, Move::Cooperate) => (3, 3), // Both cooperate
        (Move::Defect, Move::Defect) => (1, 1),       // Both defect
        (Move::Cooperate, Move::Defect) => (0, 5),    // P1 cooperates, P2 defects
        (Move::Defect, Move::Cooperate) => (5, 0),    // P1 defects, P2 cooperates
    }
}

struct Player {
    process: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout_reader: BufReader<std::process::ChildStdout>,
}

impl Player {
    fn new(binary_path: &str) -> Result<Self, String> {
        let mut process = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;
        let stdin = process.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = process.stdout.take().ok_or("Failed to open stdout")?;
        let stdout_reader = BufReader::new(stdout);
        Ok(Player {
            process,
            stdin,
            stdout_reader,
        })
    }
    fn send(&mut self, message: &str) -> Result<(), String> {
        writeln!(self.stdin, "{}", message).map_err(|_| "Failed to write to player".to_string())?;
        self.stdin
            .flush()
            .map_err(|_| "Failed to flush stdin".to_string())?;
        Ok(())
    }
    fn read_with_timeout(&mut self, timeout: Duration) -> Result<String, String> {
        let (tx, rx) = channel();
        let reader = &mut self.stdout_reader;
        thread::scope(|s| {
            s.spawn(|| {
                let mut response = String::new();
                match reader.read_line(&mut response) {
                    Ok(0) => tx
                        .send(Err("Player disconnected".to_string()))
                        .unwrap_or(()),
                    Ok(_) => tx.send(Ok(response)).unwrap_or(()),
                    Err(_) => tx.send(Err("Read error".to_string())).unwrap_or(()),
                }
            });
            match rx.recv_timeout(timeout) {
                Ok(result) => result,
                Err(RecvTimeoutError::Timeout) => Err("TLE".to_string()),
                Err(RecvTimeoutError::Disconnected) => Err("Player disconnected".to_string()),
            }
        })
    }
    fn cleanup(&mut self) {
        let _ = self.send("END");
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <player1_binary> <player2_binary>", args[0]);
        println!("RE RE");
        return;
    }
    let mut player1 = match Player::new(&args[1]) {
        Ok(p) => p,
        Err(_) => {
            println!("RE 0");
            return;
        }
    };
    let mut player2 = match Player::new(&args[2]) {
        Ok(p) => p,
        Err(_) => {
            player1.cleanup();
            println!("0 RE");
            return;
        }
    };
    let mut rng = rand::thread_rng();
    let num_rounds = rng.gen_range(100..=120);
    let mut score1 = 0;
    let mut score2 = 0;
    let mut last_move1: Option<Move> = None;
    let mut last_move2: Option<Move> = None;
    let timeout = Duration::from_secs(2);
    for round in 0..num_rounds {
        if let Some(m) = last_move2 {
            if player1.send(&format!("OPP {}", m.to_str())).is_err() {
                player1.cleanup();
                player2.cleanup();
                println!("RE {}", score2);
                return;
            }
        }
        if let Some(m) = last_move1 {
            if player2.send(&format!("OPP {}", m.to_str())).is_err() {
                player1.cleanup();
                player2.cleanup();
                println!("{} RE", score1);
                return;
            }
        }
        if player1.send("MOVE").is_err() {
            player1.cleanup();
            player2.cleanup();
            println!("RE {}", score2);
            return;
        }
        if player2.send("MOVE").is_err() {
            player1.cleanup();
            player2.cleanup();
            println!("{} RE", score1);
            return;
        }
        let response1 = match player1.read_with_timeout(timeout) {
            Ok(r) => r,
            Err(e) => {
                player1.cleanup();
                player2.cleanup();
                if e == "TLE" {
                    println!("TLE {}", score2);
                } else {
                    println!("RE {}", score2);
                }
                return;
            }
        };
        let response2 = match player2.read_with_timeout(timeout) {
            Ok(r) => r,
            Err(e) => {
                player1.cleanup();
                player2.cleanup();
                if e == "TLE" {
                    println!("{} TLE", score1);
                } else {
                    println!("{} RE", score1);
                }
                return;
            }
        };
        let move1 = match Move::from_str(&response1) {
            Some(m) => m,
            None => {
                player1.cleanup();
                player2.cleanup();
                println!("WA {}", score2);
                return;
            }
        };
        let move2 = match Move::from_str(&response2) {
            Some(m) => m,
            None => {
                player1.cleanup();
                player2.cleanup();
                println!("{} WA", score1);
                return;
            }
        };
        let (points1, points2) = calculate_scores(move1, move2);
        score1 += points1;
        score2 += points2;
        last_move1 = Some(move1);
        last_move2 = Some(move2);
    }
    player1.cleanup();
    player2.cleanup();
    println!("{} {}", score1, score2);
}
