export const defaultInteractiveFrontend = `
      <style>
        .board { display: grid; grid-template-columns: repeat(3, 100px); gap: 2px; margin: 20px auto; width: 306px; }
        .cell { width: 100px; height: 100px; background: #f0f0f0; border: 1px solid #ccc; display: flex; align-items: center; justify-content: center; font-size: 24px; cursor: pointer; }
        .cell:hover { background: #e0e0e0; }
      </style>
      <div id="status">Waiting for game to start...</div>
      <div class="board" id="board"></div>
      <script>
        for(let i=0; i<9; i++) {
          const cell = document.createElement('div');
          cell.className = 'cell';
          cell.onclick = () => window.gameAPI?.sendMove('MOVE ' + i);
          document.getElementById('board').appendChild(cell);
        }
      </script>
`;

export const defaultBackendCode = `use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Send game start messages
    println!("PLAYER_1:START X");
    println!("PLAYER_2:START O");
    stdout.flush().unwrap();

    // Process player moves
    for line in stdin.lock().lines() {
        let line = line.unwrap().trim().to_string();

        // Parse: "PLAYER_1:MOVE 4 X"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[1] == "MOVE" {
            let position: usize = parts[2].parse().unwrap_or(99);
            let player = parts[3];

            // Broadcast move to both players
            println!("PLAYER_1:MOVE {} {}", position, player);
            println!("PLAYER_2:MOVE {} {}", position, player);

            // For demo, end game after first move
            println!("PLAYER_1:END {}", player);
            println!("PLAYER_2:END {}", player);

            // Output final scores
            if player == "X" {
                println!("1 0");
            } else {
                println!("0 1");
            }
            break;
        }
        stdout.flush().unwrap();
    }
}`;
