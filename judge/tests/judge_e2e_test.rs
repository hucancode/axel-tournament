use axel_tournament_judge::{spawn_judge_server, JudgeConfig};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tokio::time::sleep;

/// E2E test for judge service:
/// 1. Create a rock_paper_scissor game with server code
/// 2. Create two submissions with player code
/// 3. Create a match in "pending" status
/// 4. Wait for judge to execute the match
/// 5. Verify match is completed with scores
#[tokio::test]
async fn judge_executes_rock_paper_scissor_match() {
    // Start judge server against the test database
    let judge_config = JudgeConfig {
        db_url: "ws://127.0.0.1:8001".to_string(),
        db_user: "root".to_string(),
        db_pass: "root".to_string(),
        db_ns: "tournament".to_string(),
        db_name: "axel".to_string(),
    };
    tracing_subscriber::fmt::init();
    let judge_handle = spawn_judge_server(judge_config);
    // Give the LIVE query a moment to subscribe
    sleep(Duration::from_secs(1)).await;

    // Connect to test database
    let db: Surreal<Client> = Surreal::new::<Ws>("localhost:8001")
        .await
        .expect("Failed to connect to test database");

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("Failed to sign in");

    db.use_ns("tournament")
        .use_db("axel")
        .await
        .expect("Failed to use namespace/database");

    println!("Connected to test database");

    // Step 1: Create a game with rock_paper_scissor server code
    let game_code = r#"
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Move {
    Rock,
    Paper,
    Scissor,
}

impl Move {
    fn from_str(s: &str) -> Option<Move> {
        match s.to_lowercase().trim() {
            "rock" => Some(Move::Rock),
            "paper" => Some(Move::Paper),
            "scissor" | "scissors" => Some(Move::Scissor),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Move::Rock => "rock",
            Move::Paper => "paper",
            Move::Scissor => "scissor",
        }
    }

    fn beats(&self, other: &Move) -> bool {
        matches!(
            (self, other),
            (Move::Rock, Move::Scissor) | (Move::Paper, Move::Rock) | (Move::Scissor, Move::Paper)
        )
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
        writeln!(self.stdin, "{}", message)
            .map_err(|_| "Failed to write to player".to_string())?;
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
                    Ok(0) => tx.send(Err("Player disconnected".to_string())).unwrap_or(()),
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

    let num_rounds: u32 = std::env::var("MATCH_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let mut score1 = 0;
    let mut score2 = 0;
    let mut last_move1: Option<Move> = None;
    let mut last_move2: Option<Move> = None;
    let timeout = Duration::from_secs(2);

    for round in 0..num_rounds {
        if round > 0 {
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

        if move1.beats(&move2) {
            score1 += 1;
        } else if move2.beats(&move1) {
            score2 += 1;
        }

        last_move1 = Some(move1);
        last_move2 = Some(move2);
    }

    if let Some(m) = last_move2 {
        let _ = player1.send(&format!("OPP {}", m.to_str()));
    }
    if let Some(m) = last_move1 {
        let _ = player2.send(&format!("OPP {}", m.to_str()));
    }

    player1.cleanup();
    player2.cleanup();
    println!("{} {}", score1, score2);
}
"#;

    // Use raw SQL query to insert game record
    let mut result = db
        .query("CREATE game SET name = $name, description = $description, game_code = $code, game_language = $lang RETURN id")
        .bind(("name", "Rock Paper Scissor"))
        .bind(("description", "RPS game for judge e2e test"))
        .bind(("code", game_code))
        .bind(("lang", "rust"))
        .await
        .expect("Failed to create game");

    #[derive(Deserialize)]
    struct GameId {
        id: Thing,
    }

    let game: Vec<GameId> = result.take(0).expect("Failed to get game from result");
    let game_id = game.first().expect("Game should be created").id.to_string();
    println!("Created game: {}", game_id);

    // Verify game can be fetched
    let game_thing: Thing = game_id.parse().expect("Failed to parse game ID");
    let mut verify = db
        .query("SELECT * FROM $game_id")
        .bind(("game_id", game_thing))
        .await
        .expect("Failed to query game");

    #[derive(Deserialize, Debug)]
    struct GameVerify {
        id: Thing,
        #[serde(default)]
        game_code: Option<String>,
        #[serde(default)]
        game_language: Option<String>,
    }

    let verified_games: Vec<GameVerify> =
        verify.take(0).expect("Failed to parse game verification");
    let verified_game = verified_games
        .first()
        .expect("Game should exist after creation");
    println!("Verified game exists: {:?}", verified_game);
    assert!(
        verified_game.game_code.is_some(),
        "Game should have game_code"
    );
    assert!(
        verified_game.game_language.is_some(),
        "Game should have game_language"
    );

    // Step 2: Create two submissions with player code
    let player1_code = r#"fn main() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "MOVE" {
            println!("rock");
        } else if input.starts_with("OPP") {
            // Do nothing
        } else if input == "END" {
            break;
        }
    }
}
"#;

    let player2_code = r#"fn main() {
    let moves = ["rock", "paper", "scissor"];
    let mut index = 0;
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "MOVE" {
            println!("{}", moves[index % 3]);
            index += 1;
        } else if input.starts_with("OPP") {
            // Do nothing
        } else if input == "END" {
            break;
        }
    }
}
"#;

    let mut result = db
        .query("CREATE submission SET game_id = $game_id, user_id = $user_id, code = $code, language = $lang, status = 'active' RETURN id")
        .bind(("game_id", game_id.clone()))
        .bind(("user_id", "user:test_user1"))
        .bind(("code", player1_code))
        .bind(("lang", "rust"))
        .await
        .expect("Failed to create submission 1");

    #[derive(Deserialize)]
    struct SubmissionId {
        id: Thing,
    }

    let submission1: Vec<SubmissionId> = result
        .take(0)
        .expect("Failed to get submission1 from result");
    let submission1_id = submission1
        .first()
        .expect("Submission 1 should be created")
        .id
        .to_string();
    println!("Created submission 1: {}", submission1_id);

    let mut result = db
        .query("CREATE submission SET game_id = $game_id, user_id = $user_id, code = $code, language = $lang, status = 'active' RETURN id")
        .bind(("game_id", game_id.clone()))
        .bind(("user_id", "user:test_user2"))
        .bind(("code", player2_code))
        .bind(("lang", "rust"))
        .await
        .expect("Failed to create submission 2");

    let submission2: Vec<SubmissionId> = result
        .take(0)
        .expect("Failed to get submission2 from result");
    let submission2_id = submission2
        .first()
        .expect("Submission 2 should be created")
        .id
        .to_string();
    println!("Created submission 2: {}", submission2_id);

    // Step 3: Create a match in "pending" status
    let mut result = db
        .query("CREATE match SET game_id = $game_id, status = 'pending', participants = $participants RETURN id")
        .bind(("game_id", game_id.clone()))
        .bind(("participants", json!([
            {
                "submission_id": submission1_id,
                "user_id": "user:test_user1"
            },
            {
                "submission_id": submission2_id,
                "user_id": "user:test_user2"
            }
        ])))
        .await
        .expect("Failed to create match");

    #[derive(Deserialize)]
    struct MatchId {
        id: Thing,
    }

    let match_record: Vec<MatchId> = result.take(0).expect("Failed to get match from result");
    let match_id = match_record
        .first()
        .expect("Match should be created")
        .id
        .to_string();
    println!("Created match: {} (status: pending)", match_id);

    // Step 4: Wait for judge to execute the match (max 60 seconds)
    println!("Waiting for judge to execute the match...");

    let match_thing: Thing = match_id.parse().expect("Failed to parse match ID as Thing");

    for attempt in 1..=60 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        #[derive(Deserialize, Debug)]
        struct MatchStatus {
            status: String,
            #[serde(default)]
            participants: Vec<MatchParticipant>,
            #[serde(default)]
            started_at: Option<String>,
            #[serde(default)]
            completed_at: Option<String>,
            #[serde(default)]
            metadata: Option<serde_json::Value>,
        }

        #[derive(Deserialize, Debug)]
        struct MatchParticipant {
            #[serde(default)]
            score: Option<f64>,
        }

        let mut result = db
            .query("SELECT status, participants, started_at, completed_at, metadata FROM $match_id")
            .bind(("match_id", match_thing.clone()))
            .await
            .expect("Failed to fetch match");

        let matches: Vec<MatchStatus> = result.take(0).expect("Failed to get match results");

        if let Some(match_data) = matches.first() {
            println!(
                "[Attempt {}/60] Match status: {}",
                attempt, match_data.status
            );

            if match_data.status == "failed" {
                println!("Match failed. Metadata: {:?}", match_data.metadata);
                let _ = judge_handle.shutdown().await;
                panic!("Match execution failed");
            } else if match_data.status == "completed" {
                println!("Match completed!");
                // Step 5: Verify the match has scores
                assert_eq!(
                    match_data.participants.len(),
                    2,
                    "Should have 2 participants"
                );
                println!("=== Match Results ===");
                let mut scores: Vec<f64> = Vec::new();
                for participant in match_data.participants.iter() {
                    let score = participant.score.expect("Score should be present");
                    scores.push(score);
                }
                // Ensure both players produced valid (non-error) scores
                assert!(
                    scores.iter().all(|score| *score > 0.0),
                    "Both players should have a positive score"
                );
                // RPS game should finish nearly tied
                let score_diff = (scores[0] - scores[1]).abs();
                assert!(
                    score_diff <= 1.0,
                    "Scores should not differ by more than 1 point, diff: {}",
                    score_diff
                );
                // Verify metadata exists
                assert!(
                    match_data.started_at.is_some(),
                    "Match should have started_at timestamp"
                );
                assert!(
                    match_data.completed_at.is_some(),
                    "Match should have completed_at timestamp"
                );
                let _ = judge_handle.shutdown().await;
                return;
            }
        }
    }

    let _ = judge_handle.shutdown().await;
    panic!("Judge did not execute the match within 60 seconds");
}
