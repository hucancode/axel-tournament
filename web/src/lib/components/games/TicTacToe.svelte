<script lang="ts">
  interface Props {
    ws: WebSocket | null;
    wsConnected: boolean;
  }

  let { ws, wsConnected }: Props = $props();

  let board = $state<(string | null)[]>(Array(9).fill(null));
  let mySymbol = $state<'X' | 'O' | null>(null);
  let gameStatus = $state<'waiting' | 'playing' | 'finished'>('waiting');
  let result = $state<string | null>(null);
  let isMyTurn = $state(false);

  function makeMove(row: number, col: number) {
    if (!wsConnected || !ws || gameStatus !== 'playing' || !isMyTurn) {
      return;
    }

    const index = row * 3 + col;
    if (board[index]) {
      return; // Cell already occupied
    }

    // Send move in text protocol format: "MOVE row col"
    ws.send(`MOVE ${row} ${col}`);

    // Optimistically update the board
    board[index] = mySymbol;
    isMyTurn = false;
  }

  function handleTextMessage(text: string) {
    const trimmed = text.trim();
    const parts = trimmed.split(/\s+/);

    if (parts.length === 0) return;

    switch (parts[0]) {
      case 'START':
        // Format: "START X" or "START O"
        if (parts.length === 2) {
          mySymbol = parts[1] as 'X' | 'O';
          gameStatus = 'playing';
          isMyTurn = mySymbol === 'X'; // X always goes first
        }
        break;

      case 'MOVE':
        // Format: "MOVE row col" - opponent's move
        if (parts.length === 3) {
          const row = parseInt(parts[1]);
          const col = parseInt(parts[2]);
          const index = row * 3 + col;
          const opponentSymbol = mySymbol === 'X' ? 'O' : 'X';
          board[index] = opponentSymbol;
          isMyTurn = true;
        }
        break;

      case 'WIN':
        gameStatus = 'finished';
        result = 'You Win!';
        break;

      case 'LOSE':
        gameStatus = 'finished';
        result = 'You Lose!';
        break;

      case 'DRAW':
        gameStatus = 'finished';
        result = 'Draw!';
        break;

      case 'INVALID':
        // Format: "INVALID message"
        alert(parts.slice(1).join(' '));
        isMyTurn = true; // Allow retry
        break;
    }
  }

  // Listen for WebSocket messages
  $effect(() => {
    if (ws) {
      const handleMessage = (event: MessageEvent) => {
        handleTextMessage(event.data);
      };

      ws.addEventListener('message', handleMessage);
      return () => ws.removeEventListener('message', handleMessage);
    }
  });
</script>

<div class="tic-tac-toe">
  <div class="game-info">
    <h3>Tic Tac Toe</h3>
    {#if gameStatus === 'waiting'}
      <p>Waiting for game to start...</p>
    {:else if gameStatus === 'playing'}
      {#if mySymbol}
        <p>You are: <strong>{mySymbol}</strong></p>
      {/if}
      <p class="turn-indicator">
        {isMyTurn ? '🟢 Your turn' : '🔴 Opponent\'s turn'}
      </p>
    {:else if gameStatus === 'finished'}
      <p class="game-result">
        {result || 'Game Over'}
      </p>
    {/if}
  </div>

  <div class="board">
    {#each Array(9) as _, index}
      {@const row = Math.floor(index / 3)}
      {@const col = index % 3}
      <button
        class="cell"
        class:disabled={!wsConnected || board[index] !== null || gameStatus !== 'playing' || !isMyTurn}
        onclick={() => makeMove(row, col)}
      >
        {board[index] || ''}
      </button>
    {/each}
  </div>

  {#if !wsConnected}
    <p class="connection-status">⚠ Connecting to game server...</p>
  {/if}
</div>

<style>
  .tic-tac-toe {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }

  .game-info {
    text-align: center;
    margin-bottom: 2rem;
  }

  .game-info h3 {
    margin: 0 0 1rem 0;
    color: #333;
  }

  .game-result {
    font-size: 1.2rem;
    font-weight: bold;
    color: #2e7d32;
  }

  .board {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px;
    background: #333;
    padding: 4px;
    border-radius: 8px;
  }

  .cell {
    width: 80px;
    height: 80px;
    background: white;
    border: none;
    font-size: 2rem;
    font-weight: bold;
    cursor: pointer;
    transition: background-color 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cell:hover:not(.disabled) {
    background: #f0f0f0;
  }

  .cell.disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .connection-status {
    margin-top: 1rem;
    color: #f57c00;
    font-style: italic;
  }
</style>
