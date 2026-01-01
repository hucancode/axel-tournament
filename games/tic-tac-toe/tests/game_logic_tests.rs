use game_framework::GameLogic;
use tic_tac_toe::game_logic::{Move, TicTacToeGame};

#[test]
fn test_parse_move() {
    // Valid moves 0-8
    for i in 0..9 {
        let mv = TicTacToeGame::parse_move(&i.to_string()).unwrap();
        assert_eq!(mv.0, i);
    }

    // With whitespace
    assert_eq!(TicTacToeGame::parse_move(" 5 ").unwrap().0, 5);
    assert_eq!(TicTacToeGame::parse_move("  0  ").unwrap().0, 0);

    // Invalid moves
    assert!(TicTacToeGame::parse_move("9").is_err()); // Out of range
    assert!(TicTacToeGame::parse_move("10").is_err());
    assert!(TicTacToeGame::parse_move("-1").is_err());
    assert!(TicTacToeGame::parse_move("abc").is_err());
    assert!(TicTacToeGame::parse_move("").is_err());
}

#[test]
fn test_make_move_basic() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 (X) moves to position 0
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap();
    assert_eq!(state.board[0], 'X');
    assert_eq!(state.current_player, 1); // Turn switches to player 1
    assert!(!state.game_over);

    // Player 1 (O) moves to position 1
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap();
    assert_eq!(state.board[1], 'O');
    assert_eq!(state.current_player, 0); // Turn switches back to player 0
    assert!(!state.game_over);
}

#[test]
fn test_cannot_move_when_not_your_turn() {
    let mut state = TicTacToeGame::new_game();

    // Try to move as player 1 when it's player 0's turn
    let result = TicTacToeGame::make_move(&mut state, 1, &Move(0));
    assert!(result.is_err());
    assert_eq!(state.board[0], ' '); // Board unchanged
}

#[test]
fn test_cannot_move_on_occupied_position() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 moves to position 4
    TicTacToeGame::make_move(&mut state, 0, &Move(4)).unwrap();

    // Player 1 tries to move to same position
    let result = TicTacToeGame::make_move(&mut state, 1, &Move(4));
    assert!(result.is_err());
}

#[test]
fn test_win_horizontal_top_row() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 (X) wins with top row: 0, 1, 2
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(3)).unwrap(); // O at 3
    TicTacToeGame::make_move(&mut state, 0, &Move(1)).unwrap(); // X at 1
    TicTacToeGame::make_move(&mut state, 1, &Move(4)).unwrap(); // O at 4
    TicTacToeGame::make_move(&mut state, 0, &Move(2)).unwrap(); // X at 2 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(0));
    assert_eq!(TicTacToeGame::get_scores(&state), vec![3, 0]); // Winner gets 3 points
}

#[test]
fn test_win_horizontal_middle_row() {
    let mut state = TicTacToeGame::new_game();

    // Player 1 (O) wins with middle row: 3, 4, 5
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(3)).unwrap(); // O at 3
    TicTacToeGame::make_move(&mut state, 0, &Move(1)).unwrap(); // X at 1
    TicTacToeGame::make_move(&mut state, 1, &Move(4)).unwrap(); // O at 4
    TicTacToeGame::make_move(&mut state, 0, &Move(6)).unwrap(); // X at 6
    TicTacToeGame::make_move(&mut state, 1, &Move(5)).unwrap(); // O at 5 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(1));
    assert_eq!(TicTacToeGame::get_scores(&state), vec![0, 3]); // Winner gets 3 points
}

#[test]
fn test_win_horizontal_bottom_row() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 (X) wins with bottom row: 6, 7, 8
    TicTacToeGame::make_move(&mut state, 0, &Move(6)).unwrap(); // X at 6
    TicTacToeGame::make_move(&mut state, 1, &Move(0)).unwrap(); // O at 0
    TicTacToeGame::make_move(&mut state, 0, &Move(7)).unwrap(); // X at 7
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap(); // O at 1
    TicTacToeGame::make_move(&mut state, 0, &Move(8)).unwrap(); // X at 8 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(0));
}

#[test]
fn test_win_vertical_left_column() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 (X) wins with left column: 0, 3, 6
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap(); // O at 1
    TicTacToeGame::make_move(&mut state, 0, &Move(3)).unwrap(); // X at 3
    TicTacToeGame::make_move(&mut state, 1, &Move(2)).unwrap(); // O at 2
    TicTacToeGame::make_move(&mut state, 0, &Move(6)).unwrap(); // X at 6 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(0));
}

#[test]
fn test_win_vertical_middle_column() {
    let mut state = TicTacToeGame::new_game();

    // Player 1 (O) wins with middle column: 1, 4, 7
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap(); // O at 1
    TicTacToeGame::make_move(&mut state, 0, &Move(2)).unwrap(); // X at 2
    TicTacToeGame::make_move(&mut state, 1, &Move(4)).unwrap(); // O at 4
    TicTacToeGame::make_move(&mut state, 0, &Move(3)).unwrap(); // X at 3
    TicTacToeGame::make_move(&mut state, 1, &Move(7)).unwrap(); // O at 7 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(1));
}

#[test]
fn test_win_vertical_right_column() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 (X) wins with right column: 2, 5, 8
    TicTacToeGame::make_move(&mut state, 0, &Move(2)).unwrap(); // X at 2
    TicTacToeGame::make_move(&mut state, 1, &Move(0)).unwrap(); // O at 0
    TicTacToeGame::make_move(&mut state, 0, &Move(5)).unwrap(); // X at 5
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap(); // O at 1
    TicTacToeGame::make_move(&mut state, 0, &Move(8)).unwrap(); // X at 8 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(0));
}

#[test]
fn test_win_diagonal_top_left_to_bottom_right() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 (X) wins with diagonal: 0, 4, 8
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap(); // O at 1
    TicTacToeGame::make_move(&mut state, 0, &Move(4)).unwrap(); // X at 4
    TicTacToeGame::make_move(&mut state, 1, &Move(2)).unwrap(); // O at 2
    TicTacToeGame::make_move(&mut state, 0, &Move(8)).unwrap(); // X at 8 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(0));
}

#[test]
fn test_win_diagonal_top_right_to_bottom_left() {
    let mut state = TicTacToeGame::new_game();

    // Player 1 (O) wins with diagonal: 2, 4, 6
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(2)).unwrap(); // O at 2
    TicTacToeGame::make_move(&mut state, 0, &Move(1)).unwrap(); // X at 1
    TicTacToeGame::make_move(&mut state, 1, &Move(4)).unwrap(); // O at 4
    TicTacToeGame::make_move(&mut state, 0, &Move(3)).unwrap(); // X at 3
    TicTacToeGame::make_move(&mut state, 1, &Move(6)).unwrap(); // O at 6 - wins!

    assert!(state.game_over);
    assert_eq!(state.winner, Some(1));
}

#[test]
fn test_draw() {
    let mut state = TicTacToeGame::new_game();

    // Create a draw scenario:
    // X O X
    // O X X
    // O X O
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap(); // X at 0
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap(); // O at 1
    TicTacToeGame::make_move(&mut state, 0, &Move(2)).unwrap(); // X at 2
    TicTacToeGame::make_move(&mut state, 1, &Move(3)).unwrap(); // O at 3
    TicTacToeGame::make_move(&mut state, 0, &Move(4)).unwrap(); // X at 4
    TicTacToeGame::make_move(&mut state, 1, &Move(6)).unwrap(); // O at 6
    TicTacToeGame::make_move(&mut state, 0, &Move(5)).unwrap(); // X at 5
    TicTacToeGame::make_move(&mut state, 1, &Move(8)).unwrap(); // O at 8
    TicTacToeGame::make_move(&mut state, 0, &Move(7)).unwrap(); // X at 7

    assert!(state.game_over);
    assert_eq!(state.winner, None); // Draw
    assert_eq!(TicTacToeGame::get_scores(&state), vec![1, 1]); // Each gets 1 point
}

#[test]
fn test_cannot_move_after_game_over() {
    let mut state = TicTacToeGame::new_game();

    // Create winning scenario
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap();
    TicTacToeGame::make_move(&mut state, 1, &Move(3)).unwrap();
    TicTacToeGame::make_move(&mut state, 0, &Move(1)).unwrap();
    TicTacToeGame::make_move(&mut state, 1, &Move(4)).unwrap();
    TicTacToeGame::make_move(&mut state, 0, &Move(2)).unwrap(); // X wins

    assert!(state.game_over);

    // Try to make another move
    let result = TicTacToeGame::make_move(&mut state, 1, &Move(5));
    assert!(result.is_err());
}

#[test]
fn test_is_game_over() {
    let mut state = TicTacToeGame::new_game();

    assert!(!TicTacToeGame::is_game_over(&state));

    // Make some moves
    TicTacToeGame::make_move(&mut state, 0, &Move(0)).unwrap();
    TicTacToeGame::make_move(&mut state, 1, &Move(1)).unwrap();
    assert!(!TicTacToeGame::is_game_over(&state));

    // Complete a winning line
    TicTacToeGame::make_move(&mut state, 0, &Move(3)).unwrap();
    TicTacToeGame::make_move(&mut state, 1, &Move(2)).unwrap();
    TicTacToeGame::make_move(&mut state, 0, &Move(6)).unwrap(); // X wins vertically

    assert!(TicTacToeGame::is_game_over(&state));
}

#[test]
fn test_get_scores() {
    let mut state = TicTacToeGame::new_game();

    // Before game over
    assert_eq!(TicTacToeGame::get_scores(&state), vec![0, 0]);

    // Player 0 wins
    state.game_over = true;
    state.winner = Some(0);
    assert_eq!(TicTacToeGame::get_scores(&state), vec![3, 0]);

    // Player 1 wins
    state.winner = Some(1);
    assert_eq!(TicTacToeGame::get_scores(&state), vec![0, 3]);

    // Draw
    state.winner = None;
    assert_eq!(TicTacToeGame::get_scores(&state), vec![1, 1]);
}

#[test]
fn test_encode_state_for_player() {
    let mut state = TicTacToeGame::new_game();

    // Player 0 gets 'X' symbol
    let encoded_p0 = TicTacToeGame::encode_state_for_player(&state, 0);
    assert!(encoded_p0.starts_with('X'));
    assert!(encoded_p0.contains('.'));  // Empty board represented as dots

    // Player 1 gets 'O' symbol
    let encoded_p1 = TicTacToeGame::encode_state_for_player(&state, 1);
    assert!(encoded_p1.starts_with('O'));

    // Make some moves and check board encoding
    state.board[0] = 'X';
    state.board[4] = 'O';
    let encoded = TicTacToeGame::encode_state_for_player(&state, 0);
    let board_str = state.board_to_string();
    assert_eq!(board_str.chars().nth(0).unwrap(), 'X');
    assert_eq!(board_str.chars().nth(4).unwrap(), 'O');
}

#[test]
fn test_board_to_string() {
    let mut state = TicTacToeGame::new_game();

    // Empty board
    assert_eq!(state.board_to_string(), ".........");

    // Board with some moves
    state.board[0] = 'X';
    state.board[4] = 'O';
    state.board[8] = 'X';
    assert_eq!(state.board_to_string(), "X...O...X");
}

#[test]
fn test_invalid_player_index() {
    let mut state = TicTacToeGame::new_game();

    // Player index 2 is invalid (only 0 and 1 are valid)
    let result = TicTacToeGame::make_move(&mut state, 2, &Move(0));
    assert!(result.is_err());
}

#[test]
fn test_get_state_message() {
    let mut state = TicTacToeGame::new_game();
    state.board[0] = 'X';
    state.current_player = 1;

    let msg = TicTacToeGame::get_state_message(&state);
    assert_eq!(msg["type"], "game_state");
    assert_eq!(msg["current_player"], 1);
    assert_eq!(msg["game_over"], false);
    assert_eq!(msg["winner"], serde_json::Value::Null);
}

#[test]
fn test_get_game_over_message() {
    let mut state = TicTacToeGame::new_game();
    state.game_over = true;
    state.winner = Some(0);

    let msg = TicTacToeGame::get_game_over_message(&state);
    assert_eq!(msg["type"], "game_over");
    assert_eq!(msg["winner"], 0);
    assert_eq!(msg["scores"], serde_json::json!([3, 0]));
}
