mod common;

use axum::http::{self, StatusCode};
use serde_json::json;

/// End-to-end test for complete tournament flow with real match execution:
/// 1. Game setter creates a rock_paper_scissor game
/// 2. Game setter opens/creates a tournament
/// 3. Two players join and submit their code
/// 4. Game setter starts the tournament (triggers match generation)
/// 5. System executes matchmaking (AllVsAll)
/// 6. Wait for judge to execute matches in Docker containers
/// 7. Verify actual scores from real gameplay and leaderboard
#[tokio::test]
async fn complete_tournament_flow_with_two_players() {
    // Setup test app with unique namespace
    let app = common::setup_app(&common::unique_name("e2e_tournament_")).await;

    // Get game setter token
    let game_setter_token = common::game_setter_token(&app).await;

    // Step 1: Game setter creates a rock_paper_scissor game
    let (game_status, game_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/games",
        Some(json!({
            "name": format!("Rock Paper Scissor {}", common::unique_name("")),
            "description": "Classic RPS game for e2e test",
            "supported_languages": ["rust", "go", "c"]
        })),
        Some(&game_setter_token),
    )
    .await;
    assert!(
        game_status == StatusCode::CREATED || game_status == StatusCode::OK,
        "Failed to create game: status {}, body {:?}",
        game_status,
        game_body
    );
    let game_id = common::extract_thing_id(&game_body["id"]);
    println!("Created game: {}", game_id);

    // Step 1b: Game setter uploads the server code (game orchestrator)
    let server_code = r#"use std::env;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::thread;
use std::time::Duration;
use rand::Rng;

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
        let mut line = String::new();
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

    let mut rng = rand::thread_rng();
    let num_rounds = rng.gen_range(100..=120);
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

    let (upload_status, upload_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/games/{}/game-code", game_id),
        Some(json!({
            "language": "rust",
            "code_content": server_code
        })),
        Some(&game_setter_token),
    )
    .await;
    assert!(
        upload_status == StatusCode::OK,
        "Failed to upload game code: status {}, body {:?}",
        upload_status,
        upload_body
    );
    println!("Uploaded server code for game");

    // Step 2: Game setter creates/opens a tournament
    let (tournament_status, tournament_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/game-setter/tournaments",
        Some(json!({
            "game_id": game_id,
            "name": format!("RPS Championship {}", common::unique_name("")),
            "description": "E2E test tournament with 2 players",
            "min_players": 2,
            "max_players": 10,
            "match_generation_type": "all_vs_all"
        })),
        Some(&game_setter_token),
    )
    .await;
    assert!(
        tournament_status == StatusCode::CREATED || tournament_status == StatusCode::OK,
        "Failed to create tournament: status {}, body {:?}",
        tournament_status,
        tournament_body
    );
    let tournament_id = common::extract_thing_id(&tournament_body["id"]);
    assert_eq!(tournament_body["status"], "registration");
    println!("Created tournament: {}", tournament_id);

    // Step 3a: Register player 1 and submit code
    let player1_email = format!("{}@test.com", common::unique_name("player1_"));
    let (_, player1_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(json!({
            "email": player1_email,
            "username": common::unique_name("player1_"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;
    let player1_token = player1_body["token"].as_str().unwrap();
    let player1_user_id = common::extract_thing_id(&player1_body["user"]["id"]);
    println!("Registered player 1: {}", player1_user_id);

    // Player 1 joins tournament
    let (join1_status, _) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/tournaments/{}/join", tournament_id),
        None,
        Some(player1_token),
    )
    .await;
    assert_eq!(join1_status, StatusCode::CREATED);
    println!("Player 1 joined tournament");

    // Player 1 submits code (simple rock strategy)
    let (submit1_status, submit1_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/submissions",
        Some(json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": r#"
fn main() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "MOVE" {
            println!("rock");
        } else if input.starts_with("OPP") {
            // Do nothing, just read opponent's move
        } else if input == "END" {
            break;
        }
    }
}
"#
        })),
        Some(player1_token),
    )
    .await;
    assert!(
        submit1_status == StatusCode::CREATED || submit1_status == StatusCode::OK,
        "Failed to submit player 1 code: status {}, body {:?}",
        submit1_status,
        submit1_body
    );
    let player1_submission_id = common::extract_thing_id(&submit1_body["id"]);
    println!("Player 1 submitted code: {}", player1_submission_id);

    // Step 3b: Register player 2 and submit code
    let player2_email = format!("{}@test.com", common::unique_name("player2_"));
    let (_, player2_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/auth/register",
        Some(json!({
            "email": player2_email,
            "username": common::unique_name("player2_"),
            "password": "password123",
            "location": "US"
        })),
        None,
    )
    .await;
    let player2_token = player2_body["token"].as_str().unwrap();
    let player2_user_id = common::extract_thing_id(&player2_body["user"]["id"]);
    println!("Registered player 2: {}", player2_user_id);

    // Player 2 joins tournament
    let (join2_status, _) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/tournaments/{}/join", tournament_id),
        None,
        Some(player2_token),
    )
    .await;
    assert_eq!(join2_status, StatusCode::CREATED);
    println!("Player 2 joined tournament");

    // Player 2 submits code (alternating strategy)
    let (submit2_status, submit2_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/submissions",
        Some(json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": r#"
fn main() {
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
"#
        })),
        Some(player2_token),
    )
    .await;
    assert!(
        submit2_status == StatusCode::CREATED || submit2_status == StatusCode::OK,
        "Failed to submit player 2 code: status {}, body {:?}",
        submit2_status,
        submit2_body
    );
    let player2_submission_id = common::extract_thing_id(&submit2_body["id"]);
    println!("Player 2 submitted code: {}", player2_submission_id);

    // Verify participants before starting tournament
    let (participants_status, participants_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/tournaments/{}/participants", tournament_id),
        None,
        None,
    )
    .await;
    assert_eq!(participants_status, StatusCode::OK);
    let participants = participants_body.as_array().unwrap();
    assert_eq!(participants.len(), 2, "Should have 2 participants");
    println!("Verified 2 participants in tournament");

    // Step 4: Game setter starts the tournament
    let (start_status, start_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/game-setter/tournaments/{}/start", tournament_id),
        None,
        Some(&game_setter_token),
    )
    .await;
    assert!(
        start_status == StatusCode::OK,
        "Failed to start tournament: status {}, body {:?}",
        start_status,
        start_body
    );
    assert_eq!(start_body["status"], "running");
    println!("Tournament started, status: running");

    // Step 5: Verify matches were generated (AllVsAll: 2x2 = 4 matches)
    let (matches_status, matches_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/matches?tournament_id={}", tournament_id),
        None,
        None,
    )
    .await;
    assert_eq!(matches_status, StatusCode::OK);
    let matches = matches_body.as_array().unwrap();
    assert_eq!(matches.len(), 4, "Should have 4 matches (2x2 AllVsAll)");
    println!("Generated {} matches", matches.len());

    // Step 6: Wait for judge to execute matches
    println!("Waiting for matches to complete...");

    let mut completed_count = 0;
    let total_matches = matches.len();

    for attempt in 0..60 {  // 5 minute timeout (60 * 5 seconds)
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let (status, body) = common::json_request(
            &app,
            http::Method::GET,
            &format!("/api/matches?tournament_id={}", tournament_id),
            None,
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let current_matches = body.as_array().unwrap();

        completed_count = current_matches
            .iter()
            .filter(|m| m["status"] == "completed")
            .count();

        println!("[Attempt {}/60] Matches completed: {}/{}", attempt + 1, completed_count, total_matches);

        if completed_count == total_matches {
            println!("All matches completed!");
            break;
        }
    }

    assert_eq!(
        completed_count, total_matches,
        "Expected all {} matches to complete within timeout", total_matches
    );

    // Step 7: Verify actual scores from real gameplay
    println!("Verifying leaderboard with real match scores...");

    let (leaderboard_status, leaderboard_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/leaderboard?tournament_id={}&limit=10", tournament_id),
        None,
        None,
    )
    .await;
    assert_eq!(leaderboard_status, StatusCode::OK);
    let leaderboard = leaderboard_body.as_array().unwrap();
    assert!(!leaderboard.is_empty(), "Leaderboard should have entries");

    // Verify leaderboard with real scores
    println!("=== Leaderboard ===");
    for entry in leaderboard {
        let rank = entry["rank"].as_u64().unwrap();
        let score = entry["score"].as_f64().unwrap();
        let username = entry["username"].as_str().unwrap();
        println!("  {}. {}: {} points", rank, username, score);

        // Scores should be non-zero from real matches
        assert!(score > 0.0, "Score should be greater than 0 from real matches");
    }

    // Verify tournament participants have updated scores
    let (final_participants_status, final_participants_body) = common::json_request(
        &app,
        http::Method::GET,
        &format!("/api/tournaments/{}/participants", tournament_id),
        None,
        None,
    )
    .await;
    assert_eq!(final_participants_status, StatusCode::OK);
    let final_participants = final_participants_body.as_array().unwrap();
    assert_eq!(final_participants.len(), 2);

    for participant in final_participants {
        let score = participant["score"].as_f64().unwrap();
        let user_id = common::extract_thing_id(&participant["user_id"]);
        println!("  Participant {}: Score = {}", user_id, score);
        // Verify scores are from real match execution
        assert!(score > 0.0, "Participant score should be greater than 0 from real matches");
    }
    println!("✅ E2E test passed with real match execution!");
}
