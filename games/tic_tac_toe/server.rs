use std::io::{self, BufRead, Write};

struct TicTacToe {
    board: [char; 9],
    current_player: char,
    game_over: bool,
}

impl TicTacToe {
    fn new() -> Self {
        Self {
            board: [' '; 9],
            current_player: 'X',
            game_over: false,
        }
    }

    fn make_move(&mut self, position: usize, player: char) -> Result<(), &'static str> {
        if self.game_over {
            return Err("Game is over");
        }
        if player != self.current_player {
            return Err("Not your turn");
        }
        if position >= 9 {
            return Err("Invalid position");
        }
        if self.board[position] != ' ' {
            return Err("Position already taken");
        }

        self.board[position] = player;
        self.current_player = if player == 'X' { 'O' } else { 'X' };
        Ok(())
    }

    fn check_winner(&self) -> Option<char> {
        let lines = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // columns
            [0, 4, 8], [2, 4, 6],             // diagonals
        ];

        for line in lines {
            if self.board[line[0]] != ' ' 
                && self.board[line[0]] == self.board[line[1]] 
                && self.board[line[1]] == self.board[line[2]] {
                return Some(self.board[line[0]]);
            }
        }
        None
    }

    fn is_draw(&self) -> bool {
        self.board.iter().all(|&cell| cell != ' ') && self.check_winner().is_none()
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game = TicTacToe::new();

    // Send game start messages
    println!("PLAYER_1:START X");
    println!("PLAYER_2:START O");
    stdout.flush().unwrap();

    for line in stdin.lock().lines() {
        let line = line.unwrap().trim().to_string();
        
        // Parse: "PLAYER_1:MOVE 4 X" or "PLAYER_2:MOVE 0 O"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[1] == "MOVE" {
            let player_id = parts[0].trim_end_matches(':');
            let position: usize = parts[2].parse().unwrap_or(99);
            let player_char = parts[3].chars().next().unwrap_or(' ');

            match game.make_move(position, player_char) {
                Ok(()) => {
                    // Broadcast move to both players
                    println!("PLAYER_1:MOVE {} {}", position, player_char);
                    println!("PLAYER_2:MOVE {} {}", position, player_char);
                    
                    // Check game end
                    if let Some(winner) = game.check_winner() {
                        game.game_over = true;
                        println!("PLAYER_1:END {}", winner);
                        println!("PLAYER_2:END {}", winner);
                        
                        // Final scores for judge
                        if winner == 'X' {
                            println!("1 0");
                        } else {
                            println!("0 1");
                        }
                        break;
                    } else if game.is_draw() {
                        game.game_over = true;
                        println!("PLAYER_1:END DRAW");
                        println!("PLAYER_2:END DRAW");
                        println!("0 0");
                        break;
                    } else {
                        // Tell players whose turn it is
                        println!("PLAYER_1:TURN {}", game.current_player);
                        println!("PLAYER_2:TURN {}", game.current_player);
                    }
                }
                Err(reason) => {
                    println!("{}:ERROR {}", player_id, reason);
                }
            }
        }
        stdout.flush().unwrap();
    }
}
