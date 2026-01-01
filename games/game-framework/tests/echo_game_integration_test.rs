use anyhow::Result;
use game_framework::GameLogic;
use serde::{Deserialize, Serialize};

/// Simple Echo game for testing
/// Players submit numbers 0-10, and their score equals their number
/// Game runs for 3 rounds
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EchoMove(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoGameState {
    pub round: u32,
    pub score_player1: i32,
    pub score_player2: i32,
    pub last_move_player1: Option<EchoMove>,
    pub last_move_player2: Option<EchoMove>,
}

pub struct EchoGame;

impl GameLogic for EchoGame {
    type Move = EchoMove;
    type GameState = EchoGameState;

    fn new_game() -> Self::GameState {
        EchoGameState {
            round: 0,
            score_player1: 0,
            score_player2: 0,
            last_move_player1: None,
            last_move_player2: None,
        }
    }

    fn parse_move(input: &str) -> Result<Self::Move> {
        let num = input
            .trim()
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid move: {}", input))?;

        if num > 10 {
            return Err(anyhow::anyhow!("Move must be 0-10, got {}", num));
        }

        Ok(EchoMove(num))
    }

    fn make_move(state: &mut Self::GameState, player_idx: usize, mv: &Self::Move) -> Result<()> {
        match player_idx {
            0 => state.last_move_player1 = Some(*mv),
            1 => state.last_move_player2 = Some(*mv),
            _ => return Err(anyhow::anyhow!("Invalid player index")),
        }

        // If both players have moved, score the round
        if let (Some(move1), Some(move2)) = (state.last_move_player1, state.last_move_player2) {
            state.score_player1 += move1.0 as i32;
            state.score_player2 += move2.0 as i32;
            state.round += 1;

            // Clear moves for next round
            state.last_move_player1 = None;
            state.last_move_player2 = None;
        }

        Ok(())
    }

    fn is_game_over(state: &Self::GameState) -> bool {
        state.round >= 3
    }

    fn get_scores(state: &Self::GameState) -> Vec<i32> {
        vec![state.score_player1, state.score_player2]
    }

    fn encode_state_for_player(state: &Self::GameState, _player_idx: usize) -> String {
        format!("ROUND {}", state.round)
    }
}

#[test]
fn test_echo_game_new_game() {
    let state = EchoGame::new_game();
    assert_eq!(state.round, 0);
    assert_eq!(state.score_player1, 0);
    assert_eq!(state.score_player2, 0);
    assert_eq!(state.last_move_player1, None);
    assert_eq!(state.last_move_player2, None);
}

#[test]
fn test_echo_game_parse_move() {
    // Valid moves
    assert_eq!(EchoGame::parse_move("0").unwrap().0, 0);
    assert_eq!(EchoGame::parse_move("5").unwrap().0, 5);
    assert_eq!(EchoGame::parse_move("10").unwrap().0, 10);
    assert_eq!(EchoGame::parse_move(" 7 ").unwrap().0, 7);

    // Invalid moves
    assert!(EchoGame::parse_move("11").is_err());
    assert!(EchoGame::parse_move("100").is_err());
    assert!(EchoGame::parse_move("abc").is_err());
    assert!(EchoGame::parse_move("-1").is_err());
}

#[test]
fn test_two_players_with_valid_moves_get_valid_scores() {
    let mut state = EchoGame::new_game();

    // Round 1: Player 1 plays 5, Player 2 plays 3
    EchoGame::make_move(&mut state, 0, &EchoMove(5)).unwrap();
    assert_eq!(state.last_move_player1, Some(EchoMove(5)));
    assert_eq!(state.round, 0); // Round not complete yet

    EchoGame::make_move(&mut state, 1, &EchoMove(3)).unwrap();
    assert_eq!(state.round, 1); // Round complete
    assert_eq!(state.score_player1, 5); // Player 1 scored 5 points
    assert_eq!(state.score_player2, 3); // Player 2 scored 3 points
    assert_eq!(state.last_move_player1, None); // Moves cleared
    assert_eq!(state.last_move_player2, None);

    // Round 2: Player 1 plays 10, Player 2 plays 7
    EchoGame::make_move(&mut state, 0, &EchoMove(10)).unwrap();
    EchoGame::make_move(&mut state, 1, &EchoMove(7)).unwrap();
    assert_eq!(state.round, 2);
    assert_eq!(state.score_player1, 15); // 5 + 10
    assert_eq!(state.score_player2, 10); // 3 + 7

    // Round 3: Player 1 plays 2, Player 2 plays 8
    EchoGame::make_move(&mut state, 0, &EchoMove(2)).unwrap();
    EchoGame::make_move(&mut state, 1, &EchoMove(8)).unwrap();
    assert_eq!(state.round, 3);
    assert_eq!(state.score_player1, 17); // 15 + 2
    assert_eq!(state.score_player2, 18); // 10 + 8

    // Game should be over
    assert!(EchoGame::is_game_over(&state));

    // Verify final scores
    let scores = EchoGame::get_scores(&state);
    assert_eq!(scores, vec![17, 18]);
}

#[test]
fn test_two_players_alternating_moves() {
    let mut state = EchoGame::new_game();

    // Test that moves must alternate (both players must move each round)
    EchoGame::make_move(&mut state, 0, &EchoMove(5)).unwrap();
    assert_eq!(state.round, 0); // Round not complete

    // Player 0 tries to move again before player 1
    EchoGame::make_move(&mut state, 0, &EchoMove(3)).unwrap();
    // This should overwrite the previous move
    assert_eq!(state.last_move_player1, Some(EchoMove(3)));
    assert_eq!(state.round, 0); // Still not complete

    // Player 1 finally moves
    EchoGame::make_move(&mut state, 1, &EchoMove(7)).unwrap();
    assert_eq!(state.round, 1); // Now complete
    assert_eq!(state.score_player1, 3); // Used the last move (3)
    assert_eq!(state.score_player2, 7);
}

#[test]
fn test_invalid_player_index() {
    let mut state = EchoGame::new_game();

    // Player index 2 is invalid (only 0 and 1 are valid)
    let result = EchoGame::make_move(&mut state, 2, &EchoMove(5));
    assert!(result.is_err());
}

#[test]
fn test_game_over_after_three_rounds() {
    let mut state = EchoGame::new_game();

    assert!(!EchoGame::is_game_over(&state));

    // Round 1
    EchoGame::make_move(&mut state, 0, &EchoMove(1)).unwrap();
    EchoGame::make_move(&mut state, 1, &EchoMove(1)).unwrap();
    assert!(!EchoGame::is_game_over(&state));

    // Round 2
    EchoGame::make_move(&mut state, 0, &EchoMove(2)).unwrap();
    EchoGame::make_move(&mut state, 1, &EchoMove(2)).unwrap();
    assert!(!EchoGame::is_game_over(&state));

    // Round 3
    EchoGame::make_move(&mut state, 0, &EchoMove(3)).unwrap();
    EchoGame::make_move(&mut state, 1, &EchoMove(3)).unwrap();
    assert!(EchoGame::is_game_over(&state));
}

#[test]
fn test_encode_state_for_player() {
    let mut state = EchoGame::new_game();

    assert_eq!(EchoGame::encode_state_for_player(&state, 0), "ROUND 0");
    assert_eq!(EchoGame::encode_state_for_player(&state, 1), "ROUND 0");

    EchoGame::make_move(&mut state, 0, &EchoMove(5)).unwrap();
    EchoGame::make_move(&mut state, 1, &EchoMove(3)).unwrap();

    assert_eq!(EchoGame::encode_state_for_player(&state, 0), "ROUND 1");
    assert_eq!(EchoGame::encode_state_for_player(&state, 1), "ROUND 1");
}

#[test]
fn test_simulated_bot_game_both_bots_output_valid_moves() {
    // Simulate a game where two bots output valid moves
    let mut state = EchoGame::new_game();

    // Simulate bot outputs
    let bot1_outputs = vec!["5", "10", "0"]; // Bot 1's moves for 3 rounds
    let bot2_outputs = vec!["3", "7", "10"]; // Bot 2's moves for 3 rounds

    for (round_idx, (output1, output2)) in bot1_outputs.iter().zip(bot2_outputs.iter()).enumerate() {
        // Bot 1 outputs a move
        let move1 = EchoGame::parse_move(output1).unwrap();
        EchoGame::make_move(&mut state, 0, &move1).unwrap();

        // Bot 2 outputs a move
        let move2 = EchoGame::parse_move(output2).unwrap();
        EchoGame::make_move(&mut state, 1, &move2).unwrap();

        // Verify round completed
        assert_eq!(state.round as usize, round_idx + 1);
    }

    // Verify game is over
    assert!(EchoGame::is_game_over(&state));

    // Verify final scores
    let expected_score1 = 5 + 10 + 0; // 15
    let expected_score2 = 3 + 7 + 10; // 20

    assert_eq!(state.score_player1, expected_score1);
    assert_eq!(state.score_player2, expected_score2);

    let scores = EchoGame::get_scores(&state);
    assert_eq!(scores, vec![15, 20]);
}

#[test]
fn test_simulated_bot_game_with_whitespace() {
    // Test that bots can output moves with whitespace and still work
    let mut state = EchoGame::new_game();

    let bot1_outputs = vec![" 5 ", "\n7\n", "  0  "];
    let bot2_outputs = vec!["3\n", " 4 ", "10"];

    for (output1, output2) in bot1_outputs.iter().zip(bot2_outputs.iter()) {
        let move1 = EchoGame::parse_move(output1).unwrap();
        EchoGame::make_move(&mut state, 0, &move1).unwrap();

        let move2 = EchoGame::parse_move(output2).unwrap();
        EchoGame::make_move(&mut state, 1, &move2).unwrap();
    }

    assert!(EchoGame::is_game_over(&state));
    assert_eq!(state.score_player1, 12); // 5 + 7 + 0
    assert_eq!(state.score_player2, 17); // 3 + 4 + 10
}

#[test]
fn test_simulated_bot_game_with_invalid_move() {
    // Test that if a bot outputs an invalid move, parsing fails
    let mut state = EchoGame::new_game();

    // Bot 1 outputs valid move
    let move1 = EchoGame::parse_move("5").unwrap();
    EchoGame::make_move(&mut state, 0, &move1).unwrap();

    // Bot 2 outputs invalid move (too large)
    let result = EchoGame::parse_move("99");
    assert!(result.is_err());

    // Game state should be unchanged (waiting for player 2's move)
    assert_eq!(state.round, 0);
    assert_eq!(state.score_player1, 0);
    assert_eq!(state.score_player2, 0);
}

#[test]
fn test_complete_bot_match_simulation() {
    // Complete simulation of a bot vs bot match
    let mut state = EchoGame::new_game();

    // Define bot strategies
    struct BotPlayer {
        moves: Vec<&'static str>,
        current_move: usize,
    }

    impl BotPlayer {
        fn new(moves: Vec<&'static str>) -> Self {
            Self { moves, current_move: 0 }
        }

        fn get_next_move(&mut self) -> &str {
            let mv = self.moves[self.current_move];
            self.current_move += 1;
            mv
        }
    }

    let mut bot1 = BotPlayer::new(vec!["10", "5", "8"]);
    let mut bot2 = BotPlayer::new(vec!["2", "9", "7"]);

    // Play 3 rounds
    for _ in 0..3 {
        // Get state for each player
        let _state_for_p1 = EchoGame::encode_state_for_player(&state, 0);
        let _state_for_p2 = EchoGame::encode_state_for_player(&state, 1);

        // Bot 1 makes move
        let bot1_output = bot1.get_next_move();
        let move1 = EchoGame::parse_move(bot1_output)
            .expect("Bot 1 should output valid move");
        EchoGame::make_move(&mut state, 0, &move1)
            .expect("Bot 1 move should be valid");

        // Bot 2 makes move
        let bot2_output = bot2.get_next_move();
        let move2 = EchoGame::parse_move(bot2_output)
            .expect("Bot 2 should output valid move");
        EchoGame::make_move(&mut state, 1, &move2)
            .expect("Bot 2 move should be valid");
    }

    // Verify game completed
    assert!(EchoGame::is_game_over(&state));

    // Verify scores are valid
    let scores = EchoGame::get_scores(&state);
    assert_eq!(scores[0], 23); // 10 + 5 + 8
    assert_eq!(scores[1], 18); // 2 + 9 + 7

    // Both scores should be non-negative and reasonable
    assert!(scores[0] >= 0);
    assert!(scores[1] >= 0);
    assert!(scores[0] <= 30); // Max possible is 10+10+10
    assert!(scores[1] <= 30);
}
