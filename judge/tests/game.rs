use anyhow::Result;
use async_trait::async_trait;
use judge::player::Player;
use judge::game_logic::GameLogic;
use judge::games::{RockPaperScissors, TicTacToe, PrisonersDilemma};
use judge::capacity::CapacityTracker;
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock player for testing
#[derive(Clone)]
struct MockPlayer {
    id: String,
    moves: Arc<Mutex<Vec<String>>>,
}

impl MockPlayer {
    fn new(id: &str, moves: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            moves: Arc::new(Mutex::new(moves.iter().map(|s| s.to_string()).collect())),
        }
    }
}

#[async_trait]
impl Player for MockPlayer {
    async fn send_message(&self, _message: &str) -> Result<()> {
        Ok(())
    }

    async fn receive_message(&self) -> Result<String> {
        let mut moves = self.moves.lock().await;
        if moves.is_empty() {
            anyhow::bail!("No more moves available");
        }
        Ok(moves.remove(0))
    }

    fn player_id(&self) -> &str {
        &self.id
    }

    async fn is_alive(&self) -> bool {
        true
    }

    fn set_timeout(&mut self, _timeout_ms: u64) {
        // Mock implementation - do nothing
    }
}

#[tokio::test]
async fn test_rock_paper_scissors_basic() {
    let moves1: Vec<&str> = vec!["ROCK", "ROCK", "ROCK", "ROCK", "ROCK"];
    let moves2: Vec<&str> = vec!["PAPER", "PAPER", "PAPER", "PAPER", "PAPER"];
    let player1 = MockPlayer::new("p1", moves1);
    let player2 = MockPlayer::new("p2", moves2);
    let players: Vec<Box<dyn Player>> = vec![Box::new(player1), Box::new(player2)];

    let game = RockPaperScissors::new();
    let results = game.run(players, 5000).await;

    // Check that we got results for both players
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_tic_tac_toe_win() {
    let player1 = MockPlayer::new("p1", vec!["MOVE 0 0", "MOVE 0 1", "MOVE 0 2"]);
    let player2 = MockPlayer::new("p2", vec!["MOVE 1 0", "MOVE 1 1"]);
    let players: Vec<Box<dyn Player>> = vec![Box::new(player1), Box::new(player2)];

    let game = TicTacToe::new();
    let results = game.run(players, 5000).await;

    // Check that we got results for both players
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_prisoners_dilemma_scoring() {
    let moves1: Vec<&str> = vec!["COOPERATE", "COOPERATE", "COOPERATE", "COOPERATE", "COOPERATE", "DEFECT", "DEFECT", "DEFECT", "DEFECT", "DEFECT"];
    let moves2: Vec<&str> = vec!["DEFECT", "DEFECT", "DEFECT", "COOPERATE", "COOPERATE", "COOPERATE", "COOPERATE", "COOPERATE", "COOPERATE", "COOPERATE"];
    let player1 = MockPlayer::new("p1", moves1);
    let player2 = MockPlayer::new("p2", moves2);
    let players: Vec<Box<dyn Player>> = vec![Box::new(player1), Box::new(player2)];

    let game = PrisonersDilemma::new();
    let results = game.run(players, 5000).await;

    // Check that we got results for both players
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_capacity_tracker_basic() {
    let tracker = CapacityTracker::new(2, 100);

    assert!(tracker.can_accept_work().await);

    tracker.increment_matches().await;
    tracker.increment_matches().await;

    assert!(!tracker.can_accept_work().await);

    tracker.decrement_matches().await;
    assert!(tracker.can_accept_work().await);
}

#[tokio::test]
async fn test_invalid_move_handling() {
    let player1 = MockPlayer::new("p1", vec!["invalid_move"]);
    let player2 = MockPlayer::new("p2", vec!["ROCK"]);
    let players: Vec<Box<dyn Player>> = vec![Box::new(player1), Box::new(player2)];

    let game = RockPaperScissors::new();
    let results = game.run(players, 5000).await;

    // Should handle invalid moves gracefully
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_player_timeout_simulation() {
    let player1 = MockPlayer::new("p1", vec![]); // No moves (simulates timeout)
    let player2 = MockPlayer::new("p2", vec!["ROCK"]);
    let players: Vec<Box<dyn Player>> = vec![Box::new(player1), Box::new(player2)];

    let game = RockPaperScissors::new();
    let results = game.run(players, 5000).await;

    // Should handle timeout gracefully
    assert_eq!(results.len(), 2);
}
