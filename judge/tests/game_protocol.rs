mod common;

use anyhow::Result;
use async_trait::async_trait;
use judge::compiler::Compiler;
use judge::players::{BotPlayer, Player};
use judge::games::{Game, GameResult, RockPaperScissors, TicTacToe, PrisonersDilemma};
use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================================
// Helper functions for GameResult
// ============================================================================

fn get_score(result: &GameResult) -> i32 {
    match result {
        GameResult::Accepted(score) => *score,
        _ => -1000, // Penalty for non-accepted results
    }
}

fn is_accepted(result: &GameResult) -> bool {
    matches!(result, GameResult::Accepted(_))
}

// ============================================================================
// Test Player - for interactive scenario testing
// Simulates direct message calls without WebSocket overhead
// ============================================================================

#[derive(Clone)]
struct TestPlayer {
    id: surrealdb::sql::Thing,
    messages_to_send: Arc<Mutex<Vec<String>>>,
    messages_received: Arc<Mutex<Vec<String>>>,
    timeout_ms: u64,
}

impl TestPlayer {
    fn new(id: &str, messages: Vec<&str>) -> Self {
        Self {
            id: format!("user:{}", id).parse().unwrap(),
            messages_to_send: Arc::new(Mutex::new(
                messages.iter().map(|s| s.to_string()).collect()
            )),
            messages_received: Arc::new(Mutex::new(Vec::new())),
            timeout_ms: 5000,
        }
    }

    async fn get_received_messages(&self) -> Vec<String> {
        self.messages_received.lock().await.clone()
    }
}

#[async_trait]
impl Player for TestPlayer {
    async fn send_message(&self, message: &str) -> Result<()> {
        self.messages_received.lock().await.push(message.to_string());
        Ok(())
    }

    async fn receive_message(&self) -> Result<String> {
        let mut messages = self.messages_to_send.lock().await;
        if messages.is_empty() {
            anyhow::bail!("TestPlayer {}: No more moves available", self.id);
        }
        Ok(messages.remove(0))
    }

    fn player_id(&self) -> &surrealdb::sql::Thing {
        &self.id
    }

    async fn is_alive(&self) -> bool {
        true
    }

    fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
}

// ============================================================================
// AUTOMATED SCENARIO TESTS
// These tests use actual Rust code compilation and BotPlayer
// ============================================================================

#[tokio::test]
async fn test_automated_rock_paper_scissors() -> Result<()> {
    // Use unique IDs to avoid collisions between test runs
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let bot1_id = format!("test_rps_bot1_{}", timestamp);
    let bot2_id = format!("test_rps_bot2_{}", timestamp);

    // Load bot code from files
    let bot1_code = std::fs::read_to_string("tests/bots/rps_rock.rs")?;
    let bot2_code = std::fs::read_to_string("tests/bots/rps_paper.rs")?;

    // Compile both bots
    let compiler = Compiler::new()?;
    println!("Compiling bot 1...");
    let binary_path1 = compiler.compile_submission(&bot1_id, "rust", &bot1_code).await?;

    println!("Compiling bot 2...");
    let binary_path2 = compiler.compile_submission(&bot2_id, "rust", &bot2_code).await?;

    // Create BotPlayers
    println!("Creating bot players...");
    let bot1 = BotPlayer::new("user:bot1".parse().unwrap(), &binary_path1).await?;
    let bot2 = BotPlayer::new("user:bot2".parse().unwrap(), &binary_path2).await?;

    let players: Vec<Box<dyn Player>> = vec![Box::new(bot1), Box::new(bot2)];

    // Run the game
    println!("Running Rock-Paper-Scissors game...");
    let game = RockPaperScissors::new();
    let game_context = common::setup_test_game_context().await;
    let results = game.run(players, 5000, game_context).await;

    // Verify results
    assert_eq!(results.len(), 2, "Should have results for both players");

    // Bot 2 (PAPER) should beat Bot 1 (ROCK) every round
    // So Bot 2 should have a higher score
    let bot1_score = get_score(&results[0]);
    let bot2_score = get_score(&results[1]);

    println!("Bot 1 (ROCK) score: {}", bot1_score);
    println!("Bot 2 (PAPER) score: {}", bot2_score);
    println!("Bot 1 result: {:?}", results[0]);
    println!("Bot 2 result: {:?}", results[1]);

    assert!(is_accepted(&results[0]), "Bot 1 should have accepted result");
    assert!(is_accepted(&results[1]), "Bot 2 should have accepted result");
    assert!(bot2_score > bot1_score, "Bot 2 (PAPER) should beat Bot 1 (ROCK)");

    Ok(())
}

#[tokio::test]
async fn test_automated_tic_tac_toe() -> Result<()> {
    // Use unique IDs to avoid collisions between test runs
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let bot1_id = format!("test_ttt_bot1_{}", timestamp);
    let bot2_id = format!("test_ttt_bot2_{}", timestamp);

    // Load bot code from files
    let bot1_code = std::fs::read_to_string("tests/bots/ttt_top_row.rs")?;
    let bot2_code = std::fs::read_to_string("tests/bots/ttt_middle_col.rs")?;

    let compiler = Compiler::new()?;
    println!("Compiling Tic-Tac-Toe bots...");
    let binary_path1 = compiler.compile_submission(&bot1_id, "rust", &bot1_code).await?;
    let binary_path2 = compiler.compile_submission(&bot2_id, "rust", &bot2_code).await?;

    println!("Creating bot players...");
    let bot1 = BotPlayer::new("user:bot1".parse().unwrap(), &binary_path1).await?;
    let bot2 = BotPlayer::new("user:bot2".parse().unwrap(), &binary_path2).await?;

    let players: Vec<Box<dyn Player>> = vec![Box::new(bot1), Box::new(bot2)];

    println!("Running Tic-Tac-Toe game...");
    let game = TicTacToe::new();
    let game_context = common::setup_test_game_context().await;
    let results = game.run(players, 30000, game_context).await;

    assert_eq!(results.len(), 2, "Should have results for both players");

    let bot1_score = get_score(&results[0]);
    let bot2_score = get_score(&results[1]);

    println!("Bot 1 (X) score: {}, result: {:?}", bot1_score, results[0]);
    println!("Bot 2 (O) score: {}, result: {:?}", bot2_score, results[1]);

    // Bot 1 should win by completing the top row
    assert!(is_accepted(&results[0]), "Bot 1 should have accepted result");
    assert!(is_accepted(&results[1]), "Bot 2 should have accepted result");
    assert!(bot1_score > bot2_score, "Bot 1 should win");

    Ok(())
}

#[tokio::test]
async fn test_automated_prisoners_dilemma() -> Result<()> {
    // Use unique IDs to avoid collisions between test runs
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let bot1_id = format!("test_pd_bot1_{}", timestamp);
    let bot2_id = format!("test_pd_bot2_{}", timestamp);

    // Load bot code from files
    let bot1_code = std::fs::read_to_string("tests/bots/pd_cooperate.rs")?;
    let bot2_code = std::fs::read_to_string("tests/bots/pd_defect.rs")?;

    let compiler = Compiler::new()?;
    println!("Compiling Prisoner's Dilemma bots...");
    let binary_path1 = compiler.compile_submission(&bot1_id, "rust", &bot1_code).await?;
    let binary_path2 = compiler.compile_submission(&bot2_id, "rust", &bot2_code).await?;

    println!("Creating bot players...");
    let bot1 = BotPlayer::new("user:bot1".parse().unwrap(), &binary_path1).await?;
    let bot2 = BotPlayer::new("user:bot2".parse().unwrap(), &binary_path2).await?;

    let players: Vec<Box<dyn Player>> = vec![Box::new(bot1), Box::new(bot2)];

    println!("Running Prisoner's Dilemma game...");
    let game = PrisonersDilemma::new();
    let game_context = common::setup_test_game_context().await;
    let results = game.run(players, 5000, game_context).await;

    assert_eq!(results.len(), 2, "Should have results for both players");

    let bot1_score = get_score(&results[0]);
    let bot2_score = get_score(&results[1]);

    println!("Bot 1 (COOPERATE) score: {}, result: {:?}", bot1_score, results[0]);
    println!("Bot 2 (DEFECT) score: {}, result: {:?}", bot2_score, results[1]);

    // Bot 2 (always defect) should score higher than Bot 1 (always cooperate)
    // When one defects and one cooperates: defector gets 5, cooperator gets 0
    assert!(is_accepted(&results[0]), "Bot 1 should have accepted result");
    assert!(is_accepted(&results[1]), "Bot 2 should have accepted result");
    assert!(bot2_score > bot1_score, "Defector should score higher");

    Ok(())
}

// ============================================================================
// INTERACTIVE SCENARIO TESTS
// These tests simulate direct function calls without WebSocket overhead
// ============================================================================

#[tokio::test]
async fn test_interactive_rock_paper_scissors() {
    // Player 1 plays: ROCK, PAPER, SCISSORS, ROCK, ROCK, ROCK, ROCK (7 moves for max rounds)
    // Player 2 plays: SCISSORS, ROCK, ROCK, PAPER, ROCK, ROCK, ROCK
    // Expected wins: P1, P2, P1, P2, Draw, Draw, Draw

    let player1 = TestPlayer::new("p1", vec!["ROCK", "PAPER", "SCISSORS", "ROCK", "ROCK", "ROCK", "ROCK"]);
    let player2 = TestPlayer::new("p2", vec!["SCISSORS", "ROCK", "ROCK", "PAPER", "ROCK", "ROCK", "ROCK"]);

    let players: Vec<Box<dyn Player>> = vec![Box::new(player1.clone()), Box::new(player2.clone())];

    let game = RockPaperScissors::new();
    let game_context = common::setup_test_game_context().await;
    let results = game.run(players, 5000, game_context).await;

    // Verify results
    assert_eq!(results.len(), 2);

    let p1_score = get_score(&results[0]);
    let p2_score = get_score(&results[1]);

    println!("Interactive RPS - Player 1 score: {}, result: {:?}", p1_score, results[0]);
    println!("Interactive RPS - Player 2 score: {}, result: {:?}", p2_score, results[1]);

    assert!(is_accepted(&results[0]), "Player 1 should have accepted result");
    assert!(is_accepted(&results[1]), "Player 2 should have accepted result");

    // Verify messages received by players
    let p1_messages = player1.get_received_messages().await;
    let p2_messages = player2.get_received_messages().await;

    // Both players should receive START message
    assert!(p1_messages.iter().any(|m| m == "START"), "Player 1 should receive START");
    assert!(p2_messages.iter().any(|m| m == "START"), "Player 2 should receive START");

    // Both players should receive SCORE and END messages
    assert!(p1_messages.iter().any(|m| m.starts_with("SCORE")), "Player 1 should receive SCORE");
    assert!(p2_messages.iter().any(|m| m.starts_with("SCORE")), "Player 2 should receive SCORE");
    assert!(p1_messages.iter().any(|m| m == "END"), "Player 1 should receive END");
    assert!(p2_messages.iter().any(|m| m == "END"), "Player 2 should receive END");

    // Both players should receive multiple ROUND messages
    let p1_round_count = p1_messages.iter().filter(|m| m.starts_with("ROUND")).count();
    let p2_round_count = p2_messages.iter().filter(|m| m.starts_with("ROUND")).count();
    assert!(p1_round_count >= 3, "Player 1 should receive at least 3 ROUND messages");
    assert!(p2_round_count >= 3, "Player 2 should receive at least 3 ROUND messages");
}

#[tokio::test]
async fn test_interactive_tic_tac_toe() {
    // Player X wins with top row: (0,0), (0,1), (0,2)
    // Player O plays: (1,0), (1,1)

    let player_x = TestPlayer::new("px", vec!["MOVE 0 0", "MOVE 0 1", "MOVE 0 2"]);
    let player_o = TestPlayer::new("po", vec!["MOVE 1 0", "MOVE 1 1"]);

    let players: Vec<Box<dyn Player>> = vec![Box::new(player_x.clone()), Box::new(player_o.clone())];

    let game = TicTacToe::new();
    let game_context = common::setup_test_game_context().await;
    let results = game.run(players, 30000, game_context).await;

    assert_eq!(results.len(), 2);

    let px_score = get_score(&results[0]);
    let po_score = get_score(&results[1]);

    println!("Interactive TTT - Player X score: {}, result: {:?}", px_score, results[0]);
    println!("Interactive TTT - Player O score: {}, result: {:?}", po_score, results[1]);

    // Player X should win
    assert!(is_accepted(&results[0]), "Player X should have accepted result");
    assert!(is_accepted(&results[1]), "Player O should have accepted result");
    assert!(px_score > po_score, "Player X should win");

    // Verify messages
    let px_messages = player_x.get_received_messages().await;
    let po_messages = player_o.get_received_messages().await;

    // Both should receive START with their symbol
    assert!(px_messages.iter().any(|m| m == "START X"), "Player X should receive START X");
    assert!(po_messages.iter().any(|m| m == "START O"), "Player O should receive START O");

    // Both should receive YOUR_TURN messages
    assert!(px_messages.iter().any(|m| m == "YOUR_TURN"), "Player X should receive YOUR_TURN");
    assert!(po_messages.iter().any(|m| m == "YOUR_TURN"), "Player O should receive YOUR_TURN");

    // Board states are sent but we don't strictly validate them here
    // The important thing is the game completed successfully with correct scores

    // Both should receive SCORE and END
    assert!(px_messages.iter().any(|m| m.starts_with("SCORE")), "Player X should receive SCORE");
    assert!(po_messages.iter().any(|m| m.starts_with("SCORE")), "Player O should receive SCORE");
    assert!(px_messages.iter().any(|m| m == "END"), "Player X should receive END");
    assert!(po_messages.iter().any(|m| m == "END"), "Player O should receive END");
}

#[tokio::test]
async fn test_interactive_prisoners_dilemma() {
    // Player 1: Cooperates first 3 rounds, then defects (13 moves for max rounds)
    // Player 2: Always defects

    let moves1 = vec!["COOPERATE", "COOPERATE", "COOPERATE", "DEFECT", "DEFECT",
                      "DEFECT", "DEFECT", "DEFECT", "DEFECT", "DEFECT",
                      "DEFECT", "DEFECT", "DEFECT"];
    let moves2 = vec!["DEFECT", "DEFECT", "DEFECT", "DEFECT", "DEFECT",
                      "DEFECT", "DEFECT", "DEFECT", "DEFECT", "DEFECT",
                      "DEFECT", "DEFECT", "DEFECT"];

    let player1 = TestPlayer::new("p1", moves1);
    let player2 = TestPlayer::new("p2", moves2);

    let players: Vec<Box<dyn Player>> = vec![Box::new(player1.clone()), Box::new(player2.clone())];

    let game = PrisonersDilemma::new();
    let game_context = common::setup_test_game_context().await;
    let results = game.run(players, 5000, game_context).await;

    assert_eq!(results.len(), 2);

    let p1_score = get_score(&results[0]);
    let p2_score = get_score(&results[1]);

    println!("Interactive PD - Player 1 score: {}, result: {:?}", p1_score, results[0]);
    println!("Interactive PD - Player 2 score: {}, result: {:?}", p2_score, results[1]);

    // Player 2 (always defect) should have higher score
    // First 3 rounds: P1=0, P2=5 each = P2 gets 15
    // Remaining rounds: Both defect = 1 each
    assert!(is_accepted(&results[0]), "Player 1 should have accepted result");
    assert!(is_accepted(&results[1]), "Player 2 should have accepted result");
    assert!(p2_score > p1_score, "Player 2 should score higher");

    // Verify messages
    let p1_messages = player1.get_received_messages().await;
    let p2_messages = player2.get_received_messages().await;

    // Both should receive START
    assert!(p1_messages.iter().any(|m| m == "START"), "Player 1 should receive START");
    assert!(p2_messages.iter().any(|m| m == "START"), "Player 2 should receive START");

    // Both should receive RESULT messages (one per round)
    let p1_result_count = p1_messages.iter().filter(|m| m.starts_with("RESULT")).count();
    let p2_result_count = p2_messages.iter().filter(|m| m.starts_with("RESULT")).count();
    assert!(p1_result_count >= 7, "Player 1 should receive at least 7 RESULT messages");
    assert!(p2_result_count >= 7, "Player 2 should receive at least 7 RESULT messages");

    // Both should receive SCORE and END
    assert!(p1_messages.iter().any(|m| m.starts_with("SCORE")), "Player 1 should receive SCORE");
    assert!(p2_messages.iter().any(|m| m.starts_with("SCORE")), "Player 2 should receive SCORE");
    assert!(p1_messages.iter().any(|m| m == "END"), "Player 1 should receive END");
    assert!(p2_messages.iter().any(|m| m == "END"), "Player 2 should receive END");
}
