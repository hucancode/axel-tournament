use game_framework::GameLogic;
use rock_paper_scissors::game_logic::{Move, RockPaperScissorsGame};

#[test]
fn test_parse_move() {
    assert_eq!(
        RockPaperScissorsGame::parse_move("rock").unwrap(),
        Move::Rock
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("ROCK").unwrap(),
        Move::Rock
    );
    assert_eq!(RockPaperScissorsGame::parse_move("r").unwrap(), Move::Rock);
    assert_eq!(RockPaperScissorsGame::parse_move("R").unwrap(), Move::Rock);

    assert_eq!(
        RockPaperScissorsGame::parse_move("paper").unwrap(),
        Move::Paper
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("PAPER").unwrap(),
        Move::Paper
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("p").unwrap(),
        Move::Paper
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("P").unwrap(),
        Move::Paper
    );

    assert_eq!(
        RockPaperScissorsGame::parse_move("scissors").unwrap(),
        Move::Scissors
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("SCISSORS").unwrap(),
        Move::Scissors
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("s").unwrap(),
        Move::Scissors
    );
    assert_eq!(
        RockPaperScissorsGame::parse_move("S").unwrap(),
        Move::Scissors
    );

    assert!(RockPaperScissorsGame::parse_move("invalid").is_err());
    assert!(RockPaperScissorsGame::parse_move("").is_err());
}

#[test]
fn test_move_beats() {
    // Rock beats scissors
    assert!(Move::Rock.beats(&Move::Scissors));
    assert!(!Move::Scissors.beats(&Move::Rock));

    // Paper beats rock
    assert!(Move::Paper.beats(&Move::Rock));
    assert!(!Move::Rock.beats(&Move::Paper));

    // Scissors beats paper
    assert!(Move::Scissors.beats(&Move::Paper));
    assert!(!Move::Paper.beats(&Move::Scissors));

    // Same moves don't beat each other
    assert!(!Move::Rock.beats(&Move::Rock));
    assert!(!Move::Paper.beats(&Move::Paper));
    assert!(!Move::Scissors.beats(&Move::Scissors));
}

#[test]
fn test_rock_beats_scissors() {
    let mut state = RockPaperScissorsGame::new_game();

    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Rock).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Scissors).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 1); // Player 1 wins
    assert_eq!(state.score_player2, 0);
}

#[test]
fn test_paper_beats_rock() {
    let mut state = RockPaperScissorsGame::new_game();

    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Paper).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Rock).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 1); // Player 1 wins
    assert_eq!(state.score_player2, 0);
}

#[test]
fn test_scissors_beats_paper() {
    let mut state = RockPaperScissorsGame::new_game();

    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Scissors).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Paper).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 1); // Player 1 wins
    assert_eq!(state.score_player2, 0);
}

#[test]
fn test_draw_rock_vs_rock() {
    let mut state = RockPaperScissorsGame::new_game();

    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Rock).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Rock).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 0); // Draw
    assert_eq!(state.score_player2, 0); // Draw
}

#[test]
fn test_draw_paper_vs_paper() {
    let mut state = RockPaperScissorsGame::new_game();

    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Paper).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Paper).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 0);
    assert_eq!(state.score_player2, 0);
}

#[test]
fn test_draw_scissors_vs_scissors() {
    let mut state = RockPaperScissorsGame::new_game();

    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Scissors).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Scissors).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 0);
    assert_eq!(state.score_player2, 0);
}

#[test]
fn test_multiple_rounds() {
    let mut state = RockPaperScissorsGame::new_game();

    // Round 1: Rock vs Scissors (Player 1 wins)
    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Rock).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Scissors).unwrap();
    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 1);
    assert_eq!(state.score_player2, 0);

    // Round 2: Paper vs Rock (Player 2 wins)
    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Rock).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Paper).unwrap();
    assert_eq!(state.round, 2);
    assert_eq!(state.score_player1, 1);
    assert_eq!(state.score_player2, 1);

    // Round 3: Paper vs Paper (Draw)
    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Paper).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Paper).unwrap();
    assert_eq!(state.round, 3);
    assert_eq!(state.score_player1, 1);
    assert_eq!(state.score_player2, 1);

    // Round 4: Scissors vs Paper (Player 1 wins)
    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Scissors).unwrap();
    RockPaperScissorsGame::make_move(&mut state, 1, &Move::Paper).unwrap();
    assert_eq!(state.round, 4);
    assert_eq!(state.score_player1, 2);
    assert_eq!(state.score_player2, 1);
}

#[test]
fn test_is_game_over() {
    let mut state = RockPaperScissorsGame::new_game();

    assert!(!RockPaperScissorsGame::is_game_over(&state));

    state.round = 50;
    assert!(!RockPaperScissorsGame::is_game_over(&state));

    state.round = 99;
    assert!(!RockPaperScissorsGame::is_game_over(&state));

    state.round = 100;
    assert!(RockPaperScissorsGame::is_game_over(&state));

    state.round = 101;
    assert!(RockPaperScissorsGame::is_game_over(&state));
}

#[test]
fn test_get_scores() {
    let mut state = RockPaperScissorsGame::new_game();
    state.score_player1 = 15;
    state.score_player2 = 10;

    let scores = RockPaperScissorsGame::get_scores(&state);
    assert_eq!(scores, vec![15, 10]);
}

#[test]
fn test_encode_state_for_player() {
    let state = RockPaperScissorsGame::new_game();

    // For RPS, state is just "READY" (no history dependency)
    let state_p1 = RockPaperScissorsGame::encode_state_for_player(&state, 0);
    let state_p2 = RockPaperScissorsGame::encode_state_for_player(&state, 1);

    assert_eq!(state_p1, "READY");
    assert_eq!(state_p2, "READY");
}

#[test]
fn test_invalid_player_index() {
    let mut state = RockPaperScissorsGame::new_game();

    let result = RockPaperScissorsGame::make_move(&mut state, 2, &Move::Rock);
    assert!(result.is_err());

    let result = RockPaperScissorsGame::make_move(&mut state, 99, &Move::Paper);
    assert!(result.is_err());
}

#[test]
fn test_get_round_result_message() {
    let mut state = RockPaperScissorsGame::new_game();
    state.score_player1 = 1;
    state.score_player2 = 0;

    // Player 1 plays rock, Player 2 plays scissors (Player 1 wins)
    let result = RockPaperScissorsGame::get_round_result_message(&state, &Move::Rock, &Move::Scissors);

    assert!(result.is_some());
    let json = result.unwrap();
    assert_eq!(json["your_move"], "rock");
    assert_eq!(json["opponent_move"], "scissors");
    assert_eq!(json["result"], "win");
    assert_eq!(json["your_score"], 1);
    assert_eq!(json["opponent_score"], 0);
}

#[test]
fn test_partial_round() {
    let mut state = RockPaperScissorsGame::new_game();

    // Only player 1 moves
    RockPaperScissorsGame::make_move(&mut state, 0, &Move::Rock).unwrap();
    assert_eq!(state.last_move_player1, Some(Move::Rock));
    assert_eq!(state.round, 0); // Round not complete
    assert_eq!(state.score_player1, 0);
    assert_eq!(state.score_player2, 0);
}
