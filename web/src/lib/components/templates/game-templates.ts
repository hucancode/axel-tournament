export const defaultInteractiveFrontend = `
      <style>
        #status { text-align: center; font-size: 20px; margin: 20px; color: #fff; font-weight: bold; }
        .board { display: grid; grid-template-columns: repeat(3, 100px); gap: 2px; margin: 20px auto; width: 306px; }
        .cell { width: 100px; height: 100px; background: #f0f0f0; border: 1px solid #ccc; display: flex; align-items: center; justify-content: center; font-size: 36px; cursor: pointer; font-weight: bold; }
        .cell:hover { background: #e0e0e0; }
        .my-turn { color: #4caf50; }
        .their-turn { color: #ff9800; }
        .game-over { color: #f44336; }
      </style>
      <div id="status">Connecting to game server...</div>
      <div class="board" id="board"></div>
      <script>
        let mySymbol = '';
        let currentTurn = '';
        let gameOver = false;
        const cells = [];
        const board = ['', '', '', '', '', '', '', '', ''];

        // Create board
        for(let i=0; i<9; i++) {
          const cell = document.createElement('div');
          cell.className = 'cell';
          cell.dataset.index = i;
          cell.onclick = () => {
            if (!gameOver && currentTurn === mySymbol && board[i] === '') {
              window.gameAPI?.sendMove('MOVE ' + i + ' ' + mySymbol);
            }
          };
          cells.push(cell);
          document.getElementById('board').appendChild(cell);
        }

        function updateStatus(text, className = '') {
          const status = document.getElementById('status');
          status.textContent = text;
          status.className = className;
        }

        function onMessage(message) {
          console.log('Game received:', message);
          const parts = message.split(' ');

          if (parts[0] === 'START') {
            // START X or START O
            mySymbol = parts[1];
            currentTurn = 'X'; // X always goes first
            updateStatus(
              mySymbol === 'X' ? 'Your turn (You are X)' : 'Waiting for opponent (You are O)',
              mySymbol === 'X' ? 'my-turn' : 'their-turn'
            );
          }
          else if (parts[0] === 'MOVE') {
            // MOVE 4 X
            const position = parseInt(parts[1]);
            const symbol = parts[2];
            board[position] = symbol;
            cells[position].textContent = symbol;

            // Switch turn
            currentTurn = currentTurn === 'X' ? 'O' : 'X';
            updateStatus(
              currentTurn === mySymbol ? 'Your turn!' : 'Opponent is thinking...',
              currentTurn === mySymbol ? 'my-turn' : 'their-turn'
            );
          }
          else if (parts[0] === 'END') {
            // END X (X won)
            gameOver = true;
            const winner = parts[1];
            if (winner === mySymbol) {
              updateStatus('🎉 You won!', 'game-over my-turn');
            } else {
              updateStatus('You lost. Better luck next time!', 'game-over their-turn');
            }
          }
        }

        // Register message handler
        if (window.gameAPI) {
          window.gameAPI.onMessage(onMessage);
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
