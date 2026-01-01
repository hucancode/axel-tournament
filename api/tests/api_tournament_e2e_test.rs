mod common;

use axum::http::{self, StatusCode};
use serde_json::json;

// Simple rock-paper-scissors bot that always plays Rock
const RPS_PLAYER_ROCK: &str = r#"
fn main() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("rock");
    }
}
"#;

// Simple rock-paper-scissors bot that cycles through moves
const RPS_PLAYER_CYCLE: &str = r#"
fn main() {
    let moves = ["rock", "paper", "scissors"];
    let mut index = 0;
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("{}", moves[index % 3]);
        index += 1;
    }
}
"#;

/// End-to-end test for complete tournament flow with real match execution:
/// 1. Admin creates a tournament for rock-paper-scissors game (hardcoded game)
/// 2. Two players join and submit their code
/// 3. Admin starts the tournament (triggers match generation)
/// 4. System executes matchmaking (AllVsAll)
/// 5. Wait for game server to execute matches
/// 6. Verify actual scores from real gameplay and leaderboard
#[tokio::test]
async fn complete_tournament_flow_with_two_players() {
    // Setup test app - uses production config from environment
    let app = common::setup_app().await;

    // Get admin token for tournament management
    let admin_token = common::admin_token(&app).await;

    // Use hardcoded rock-paper-scissors game (maintained by developers)
    let game_id = "rock-paper-scissors";
    println!("Using hardcoded game: {}", game_id);

    // Step 1: Admin creates a tournament
    let (tournament_status, tournament_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/admin/tournaments",
        Some(json!({
            "game_id": game_id,
            "name": format!("RPS Championship {}", common::unique_name("")),
            "description": "E2E test tournament with 2 players",
            "min_players": 2,
            "max_players": 10,
            "match_generation_type": "all_vs_all"
        })),
        Some(&admin_token),
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

    // Step 2a: Register player 1 and submit code
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
            "code": RPS_PLAYER_ROCK
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

    // Step 2b: Register player 2 and submit code
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

    // Player 2 submits code (cycling strategy)
    let (submit2_status, submit2_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/submissions",
        Some(json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": RPS_PLAYER_CYCLE
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

    // Step 3: Admin starts the tournament
    let (start_status, start_body) = common::json_request(
        &app,
        http::Method::POST,
        &format!("/api/admin/tournaments/{}/start", tournament_id),
        None,
        Some(&admin_token),
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

    // Step 4: Verify matches were generated (AllVsAll: 2x2 = 4 matches)
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

    // Step 5: Wait for game server to execute matches
    println!("Waiting for matches to complete...");

    let mut completed_count = 0;
    let total_matches = matches.len();

    for attempt in 0..60 {
        // 5 minute timeout (60 * 5 seconds)
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let (status, body) = common::json_request(
            &app,
            http::Method::GET,
            &format!("/api/matches?tournament_id={}", tournament_id),
            None,
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        let current_matches = body.as_array().unwrap();

        completed_count = current_matches
            .iter()
            .filter(|m| m["status"] == "completed")
            .count();

        println!(
            "[Attempt {}/60] Matches completed: {}/{}",
            attempt + 1,
            completed_count,
            total_matches
        );

        if completed_count == total_matches {
            println!("All matches completed!");
            break;
        }
    }

    assert_eq!(
        completed_count, total_matches,
        "Expected all {} matches to complete within timeout",
        total_matches
    );

    // Step 6: Verify actual scores from real gameplay
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
        assert!(
            score > 0.0,
            "Score should be greater than 0 from real matches"
        );
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

    let mut scores: Vec<f64> = Vec::new();
    for participant in final_participants {
        let score = participant["score"].as_f64().unwrap();
        scores.push(score);
        let user_id = common::extract_thing_id(&participant["user_id"]);
        println!("  Participant {}: Score = {}", user_id, score);
        // Verify scores are from real match execution
        assert!(
            score > 0.0,
            "Participant score should be greater than 0 from real matches"
        );
    }
    // Rock vs. cycling strategy should end up nearly tied.
    let score_diff = (scores[0] - scores[1]).abs();
    assert!(
        score_diff <= 20.0,
        "Expected players to end near a tie, got diff {} with scores {:?}",
        score_diff,
        scores
    );
}
