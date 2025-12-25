<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { CreateGameRequest, GameType, ProgrammingLanguage } from '$lib/types';
  
  let gameType = $state<GameType>('automated');
  let name = $state('');
  let description = $state('');
  let supportedLanguages = $state<ProgrammingLanguage[]>(['rust']);
  let gameCode = $state('');
  let frontendCode = $state('');
  let gameLanguage = $state<ProgrammingLanguage>('rust');
  let roundsPerMatch = $state(10);
  let repetitions = $state(1);
  let timeoutMs = $state(1000);
  let cpuLimit = $state(0.5);
  let turnTimeoutMs = $state(100);
  let memoryLimitMb = $state(64);
  
  let creating = $state(false);
  let error = $state('');
  
  const languages: ProgrammingLanguage[] = ['rust', 'go', 'c'];
  
  const defaultInteractiveFrontend = `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>My Interactive Game</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        .game-board {
            display: grid;
            grid-template-columns: repeat(3, 100px);
            gap: 2px;
            margin: 20px auto;
            width: 306px;
        }
        .cell {
            width: 100px;
            height: 100px;
            background: #f0f0f0;
            border: 1px solid #ccc;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            cursor: pointer;
        }
        .cell:hover { background: #e0e0e0; }
        #status { text-align: center; margin: 20px; }
    </style>
</head>
<body>
    <div id="status">Waiting for game to start...</div>
    <div class="game-board" id="board"></div>

    <script>
        // Game API provided by the platform
        const gameAPI = window.gameAPI;
        
        let gameState = {};
        let myPlayer = '';
        
        // Initialize your game UI here
        function initGame() {
            const board = document.getElementById('board');
            board.innerHTML = '';
            for (let i = 0; i < 9; i++) {
                const cell = document.createElement('div');
                cell.className = 'cell';
                cell.dataset.index = i;
                cell.onclick = () => makeMove(i);
                board.appendChild(cell);
            }
        }
        
        // Handle player move
        function makeMove(index) {
            // Send move to backend
            gameAPI.sendMove(\`MOVE \${index} \${myPlayer}\`);
        }
        
        // Handle messages from backend
        gameAPI.onMessage((message) => {
            const parts = message.split(' ');
            const command = parts[0];
            
            switch (command) {
                case 'START':
                    myPlayer = parts[1];
                    document.getElementById('status').textContent = 
                        \`Game started! You are \${myPlayer}\`;
                    break;
                case 'MOVE':
                    // Update game state based on move
                    console.log('Move made:', message);
                    break;
                case 'END':
                    document.getElementById('status').textContent = 
                        \`Game ended! Winner: \${parts[1]}\`;
                    break;
            }
        });
        
        // Initialize when page loads
        initGame();
    </script>
</body>
</html>`;

  const defaultBackendCode = `use std::io::{self, BufRead, Write};

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
  
  onMount(() => {
    if (gameType === 'interactive') {
      frontendCode = defaultInteractiveFrontend;
      gameCode = defaultBackendCode;
    }
  });
  
  function onGameTypeChange() {
    if (gameType === 'interactive' && !frontendCode) {
      frontendCode = defaultInteractiveFrontend;
      gameCode = defaultBackendCode;
    }
  }
  
  function toggleLanguage(lang: ProgrammingLanguage) {
    if (supportedLanguages.includes(lang)) {
      supportedLanguages = supportedLanguages.filter(l => l !== lang);
    } else {
      supportedLanguages = [...supportedLanguages, lang];
    }
  }
  
  async function createGame() {
    if (!name.trim() || !description.trim() || !gameCode.trim()) {
      error = 'Please fill in all required fields';
      return;
    }
    
    if (gameType === 'interactive' && !frontendCode.trim()) {
      error = 'Frontend code is required for interactive games';
      return;
    }
    
    if (supportedLanguages.length === 0) {
      error = 'Please select at least one supported language';
      return;
    }
    
    creating = true;
    error = '';
    
    try {
      const request: CreateGameRequest = {
        name: name.trim(),
        description: description.trim(),
        game_type: gameType,
        supported_languages: supportedLanguages,
        game_code: gameCode.trim(),
        game_language: gameLanguage,
        frontend_code: gameType === 'interactive' ? frontendCode.trim() : undefined,
        rounds_per_match: roundsPerMatch,
        repetitions: repetitions,
        timeout_ms: timeoutMs,
        cpu_limit: cpuLimit,
        turn_timeout_ms: turnTimeoutMs,
        memory_limit_mb: memoryLimitMb,
      };
      
      await api.post('/api/admin/games', request, true);
      goto('/game-setter');
    } catch (e: any) {
      error = e.message || 'Failed to create game';
    } finally {
      creating = false;
    }
  }
</script>

<div class="create-game">
  <div class="header">
    <h1>Create New Game</h1>
    <button class="btn-secondary" onclick={() => goto('/game-setter')}>
      Back to Dashboard
    </button>
  </div>
  
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  
  <form onsubmit|preventDefault={createGame}>
    <div class="form-section">
      <h2>Basic Information</h2>
      
      <div class="form-group">
        <label for="game-type">Game Type</label>
        <select id="game-type" bind:value={gameType} onchange={onGameTypeChange}>
          <option value="automated">Automated (Code vs Code)</option>
          <option value="interactive">Interactive (Human vs Human)</option>
        </select>
      </div>
      
      <div class="form-group">
        <label for="name">Game Name</label>
        <input id="name" type="text" bind:value={name} placeholder="Enter game name" required />
      </div>
      
      <div class="form-group">
        <label for="description">Description</label>
        <textarea id="description" bind:value={description} placeholder="Describe your game" required></textarea>
      </div>
      
      <div class="form-group">
        <label>Supported Languages</label>
        <div class="checkbox-group">
          {#each languages as lang}
            <label class="checkbox-label">
              <input
                type="checkbox"
                checked={supportedLanguages.includes(lang)}
                onchange={() => toggleLanguage(lang)}
              />
              {lang.toUpperCase()}
            </label>
          {/each}
        </div>
      </div>
    </div>
    
    <div class="form-section">
      <h2>Game Code</h2>
      
      <div class="form-group">
        <label for="game-language">Backend Language</label>
        <select id="game-language" bind:value={gameLanguage}>
          {#each languages as lang}
            <option value={lang}>{lang.toUpperCase()}</option>
          {/each}
        </select>
      </div>
      
      <div class="form-group">
        <label for="game-code">Backend Code (Game Logic)</label>
        <textarea
          id="game-code"
          bind:value={gameCode}
          placeholder="Enter your game server code"
          rows="15"
          required
        ></textarea>
      </div>
      
      {#if gameType === 'interactive'}
        <div class="form-group">
          <label for="frontend-code">Frontend Code (HTML/CSS/JS)</label>
          <textarea
            id="frontend-code"
            bind:value={frontendCode}
            placeholder="Enter your game frontend code"
            rows="20"
            required
          ></textarea>
        </div>
      {/if}
    </div>
    
    <div class="form-section">
      <h2>Game Settings</h2>
      
      <div class="form-row">
        <div class="form-group">
          <label for="rounds">Rounds per Match</label>
          <input id="rounds" type="number" bind:value={roundsPerMatch} min="1" max="100" />
        </div>
        
        <div class="form-group">
          <label for="repetitions">Repetitions</label>
          <input id="repetitions" type="number" bind:value={repetitions} min="1" max="100" />
        </div>
      </div>
      
      <div class="form-row">
        <div class="form-group">
          <label for="timeout">Timeout (ms)</label>
          <input id="timeout" type="number" bind:value={timeoutMs} min="100" max="5000" />
        </div>
        
        <div class="form-group">
          <label for="cpu-limit">CPU Limit</label>
          <input id="cpu-limit" type="number" bind:value={cpuLimit} min="0.1" max="64" step="0.1" />
        </div>
      </div>
      
      <div class="form-row">
        <div class="form-group">
          <label for="turn-timeout">Turn Timeout (ms)</label>
          <input id="turn-timeout" type="number" bind:value={turnTimeoutMs} min="1" max="2000" />
        </div>
        
        <div class="form-group">
          <label for="memory-limit">Memory Limit (MB)</label>
          <input id="memory-limit" type="number" bind:value={memoryLimitMb} min="1" max="8192" />
        </div>
      </div>
    </div>
    
    <div class="form-actions">
      <button type="button" class="btn-secondary" onclick={() => goto('/game-setter')}>
        Cancel
      </button>
      <button type="submit" class="btn-primary" disabled={creating}>
        {creating ? 'Creating...' : 'Create Game'}
      </button>
    </div>
  </form>
</div>

<style>
  .create-game {
    max-width: 1000px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }
  
  .form-section {
    margin-bottom: 2rem;
    padding: 1.5rem;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
  }
  
  .form-section h2 {
    margin: 0 0 1rem 0;
    color: #333;
  }
  
  .form-group {
    margin-bottom: 1rem;
  }
  
  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
  }
  
  .form-group input,
  .form-group select,
  .form-group textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: inherit;
  }
  
  .form-group textarea {
    font-family: 'Courier New', monospace;
    resize: vertical;
  }
  
  .checkbox-group {
    display: flex;
    gap: 1rem;
  }
  
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: normal;
  }
  
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 2rem;
  }
  
  .btn-primary, .btn-secondary {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }
  
  .btn-primary {
    background: #1976d2;
    color: white;
  }
  
  .btn-primary:disabled {
    background: #ccc;
    cursor: not-allowed;
  }
  
  .btn-secondary {
    background: #f5f5f5;
    color: #333;
    border: 1px solid #ddd;
  }
  
  .error-message {
    background: #ffebee;
    color: #c62828;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }
</style>
