mod common;

use axum::http::{self, StatusCode};
use serde_json::json;

const RPS_SERVER_CODE: &str = include_str!("../../games/rock_paper_scissor/server.rs");
const RPS_PLAYER_ROCK: &str = include_str!("../../games/rock_paper_scissor/client_rock.rs");
const RPS_PLAYER_ALT: &str = include_str!("../../games/rock_paper_scissor/client_cycle.rs");

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
    // Ensure we use the same DB namespace/database defaults as the running judge/database.
    let db_namespace = std::env::var("DATABASE_NS").unwrap_or_else(|_| "axel".to_string());
    // Setup test app with shared namespace so the judge process can see the same matches
    let app = common::setup_app(&db_namespace).await;
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
            "supported_languages": ["rust", "go", "c"],
            "game_code": RPS_SERVER_CODE,
            "game_language": "rust",
            "rounds_per_match": 3,
            "repetitions": 1,
            "timeout_seconds": 120,
            "cpu_limit": "1.0",
            "memory_limit": "512m"
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
    let player1_code = RPS_PLAYER_ROCK;
    let (submit1_status, submit1_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/submissions",
        Some(json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": player1_code
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
    let player2_code = RPS_PLAYER_ALT;
    let (submit2_status, submit2_body) = common::json_request(
        &app,
        http::Method::POST,
        "/api/submissions",
        Some(json!({
            "tournament_id": tournament_id,
            "language": "rust",
            "code": player2_code
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
