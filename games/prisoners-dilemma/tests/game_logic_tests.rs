use game_framework::GameLogic;
use prisoners_dilemma::game_logic::{Move, PrisonersDilemmaGame};

#[test]
fn test_parse_move() {
    assert_eq!(
        PrisonersDilemmaGame::parse_move("cooperate").unwrap(),
        Move::Cooperate
    );
    assert_eq!(
        PrisonersDilemmaGame::parse_move("COOPERATE").unwrap(),
        Move::Cooperate
    );
    assert_eq!(
        PrisonersDilemmaGame::parse_move("c").unwrap(),
        Move::Cooperate
    );
    assert_eq!(
        PrisonersDilemmaGame::parse_move("C").unwrap(),
        Move::Cooperate
    );

    assert_eq!(
        PrisonersDilemmaGame::parse_move("defect").unwrap(),
        Move::Defect
    );
    assert_eq!(
        PrisonersDilemmaGame::parse_move("DEFECT").unwrap(),
        Move::Defect
    );
    assert_eq!(
        PrisonersDilemmaGame::parse_move("d").unwrap(),
        Move::Defect
    );
    assert_eq!(
        PrisonersDilemmaGame::parse_move("D").unwrap(),
        Move::Defect
    );

    assert!(PrisonersDilemmaGame::parse_move("invalid").is_err());
    assert!(PrisonersDilemmaGame::parse_move("").is_err());
}

#[test]
fn test_both_cooperate() {
    let mut state = PrisonersDilemmaGame::new_game();

    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Cooperate).unwrap();
    assert_eq!(state.last_move_player1, Some(Move::Cooperate));
    assert_eq!(state.round, 0); // Round not complete yet

    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Cooperate).unwrap();
    assert_eq!(state.round, 1); // Round complete
    assert_eq!(state.score_player1, 3);
    assert_eq!(state.score_player2, 3);
    // Moves are cleared after scoring
    assert_eq!(state.last_move_player1, None);
    assert_eq!(state.last_move_player2, None);
}

#[test]
fn test_both_defect() {
    let mut state = PrisonersDilemmaGame::new_game();

    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Defect).unwrap();
    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Defect).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 1);
    assert_eq!(state.score_player2, 1);
}

#[test]
fn test_player1_defects_player2_cooperates() {
    let mut state = PrisonersDilemmaGame::new_game();

    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Defect).unwrap();
    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Cooperate).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 5); // Defector wins
    assert_eq!(state.score_player2, 0); // Cooperator loses
}

#[test]
fn test_player1_cooperates_player2_defects() {
    let mut state = PrisonersDilemmaGame::new_game();

    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Cooperate).unwrap();
    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Defect).unwrap();

    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 0); // Cooperator loses
    assert_eq!(state.score_player2, 5); // Defector wins
}

#[test]
fn test_multiple_rounds() {
    let mut state = PrisonersDilemmaGame::new_game();

    // Round 1: Both cooperate
    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Cooperate).unwrap();
    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Cooperate).unwrap();
    assert_eq!(state.round, 1);
    assert_eq!(state.score_player1, 3);
    assert_eq!(state.score_player2, 3);

    // Round 2: Player 1 defects, Player 2 cooperates
    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Defect).unwrap();
    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Cooperate).unwrap();
    assert_eq!(state.round, 2);
    assert_eq!(state.score_player1, 8); // 3 + 5
    assert_eq!(state.score_player2, 3); // 3 + 0

    // Round 3: Both defect
    PrisonersDilemmaGame::make_move(&mut state, 0, &Move::Defect).unwrap();
    PrisonersDilemmaGame::make_move(&mut state, 1, &Move::Defect).unwrap();
    assert_eq!(state.round, 3);
    assert_eq!(state.score_player1, 9); // 8 + 1
    assert_eq!(state.score_player2, 4); // 3 + 1
}

#[test]
fn test_is_game_over() {
    let mut state = PrisonersDilemmaGame::new_game();

    assert!(!PrisonersDilemmaGame::is_game_over(&state));

    state.round = 50;
    assert!(!PrisonersDilemmaGame::is_game_over(&state));

    state.round = 99;
    assert!(!PrisonersDilemmaGame::is_game_over(&state));

    state.round = 100;
    assert!(PrisonersDilemmaGame::is_game_over(&state));

    state.round = 101;
    assert!(PrisonersDilemmaGame::is_game_over(&state));
}

#[test]
fn test_get_scores() {
    let mut state = PrisonersDilemmaGame::new_game();
    state.score_player1 = 42;
    state.score_player2 = 37;

    let scores = PrisonersDilemmaGame::get_scores(&state);
    assert_eq!(scores, vec![42, 37]);
}

#[test]
fn test_encode_state_for_player() {
    let mut state = PrisonersDilemmaGame::new_game();

    // First round, no previous moves
    let state_p1 = PrisonersDilemmaGame::encode_state_for_player(&state, 0);
    let state_p2 = PrisonersDilemmaGame::encode_state_for_player(&state, 1);
    assert_eq!(state_p1, "MOVE");
    assert_eq!(state_p2, "MOVE");

    // After player 1 moves
    state.last_move_player1 = Some(Move::Cooperate);
    let state_p2 = PrisonersDilemmaGame::encode_state_for_player(&state, 1);
    assert_eq!(state_p2, "OPP cooperate");

    // After player 2 moves
    state.last_move_player2 = Some(Move::Defect);
    let state_p1 = PrisonersDilemmaGame::encode_state_for_player(&state, 0);
    assert_eq!(state_p1, "OPP defect");
}

#[test]
fn test_invalid_player_index() {
    let mut state = PrisonersDilemmaGame::new_game();

    let result = PrisonersDilemmaGame::make_move(&mut state, 2, &Move::Cooperate);
    assert!(result.is_err());

    let result = PrisonersDilemmaGame::make_move(&mut state, 99, &Move::Defect);
    assert!(result.is_err());
}

#[test]
fn test_get_round_result_message() {
    let mut state = PrisonersDilemmaGame::new_game();
    state.score_player1 = 3;
    state.score_player2 = 3;

    let result =
        PrisonersDilemmaGame::get_round_result_message(&state, &Move::Cooperate, &Move::Defect);

    assert!(result.is_some());
    let json = result.unwrap();
    assert_eq!(json["your_move"], "cooperate");
    assert_eq!(json["opponent_move"], "defect");
    assert_eq!(json["your_points"], 0);
    assert_eq!(json["opponent_points"], 5);
}
