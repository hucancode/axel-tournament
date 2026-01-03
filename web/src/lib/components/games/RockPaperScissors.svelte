<script lang="ts">
  interface Props {
    ws: WebSocket | null;
    wsConnected: boolean;
  }

  let { ws, wsConnected }: Props = $props();

  let myChoice = $state<'ROCK' | 'PAPER' | 'SCISSORS' | null>(null);
  let opponentChoice = $state<'ROCK' | 'PAPER' | 'SCISSORS' | null>(null);
  let gameStatus = $state<'waiting' | 'playing' | 'finished'>('waiting');
  let roundResult = $state<string | null>(null);
  let totalRounds = $state(5);
  let scores = $state<{player: number, opponent: number}>({player: 0, opponent: 0});

  const choices = [
    { value: 'ROCK', emoji: '🪨', label: 'Rock' },
    { value: 'PAPER', emoji: '📄', label: 'Paper' },
    { value: 'SCISSORS', emoji: '✂️', label: 'Scissors' }
  ] as const;

  function makeChoice(choice: 'ROCK' | 'PAPER' | 'SCISSORS') {
    if (!wsConnected || !ws || gameStatus !== 'playing' || myChoice !== null) {
      return;
    }

    myChoice = choice;
    // Send move in text protocol format: "ROCK", "PAPER", or "SCISSORS"
    ws.send(choice);
  }

  function handleTextMessage(text: string) {
    const trimmed = text.trim();
    const parts = trimmed.split(/\s+/);

    if (parts.length === 0) return;

    switch (parts[0]) {
      case 'START':
        // Format: "START" or "START 5" (number of rounds)
        if (parts.length === 2) {
          totalRounds = parseInt(parts[1]);
        }
        gameStatus = 'playing';
        break;

      case 'RESULT':
        // Format: "RESULT your_choice opp_choice outcome"
        // Example: "RESULT ROCK SCISSORS WIN"
        if (parts.length === 4) {
          myChoice = parts[1] as 'ROCK' | 'PAPER' | 'SCISSORS';
          opponentChoice = parts[2] as 'ROCK' | 'PAPER' | 'SCISSORS';
          roundResult = parts[3]; // WIN, LOSE, or DRAW

          // Reset after 3 seconds
          setTimeout(() => {
            myChoice = null;
            opponentChoice = null;
            roundResult = null;
          }, 3000);
        }
        break;

      case 'SCORE':
        // Format: "SCORE your_score opp_score"
        if (parts.length === 3) {
          scores = {
            player: parseInt(parts[1]),
            opponent: parseInt(parts[2])
          };
        }
        break;

      case 'END':
        // Format: "END your_score opp_score"
        if (parts.length === 3) {
          scores = {
            player: parseInt(parts[1]),
            opponent: parseInt(parts[2])
          };
        }
        gameStatus = 'finished';
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

<div class="rock-paper-scissors">
  <div class="game-info">
    <h3>Rock Paper Scissors</h3>
    {#if gameStatus === 'waiting'}
      <p>Waiting for players...</p>
    {:else if gameStatus === 'playing'}
      <div class="game-status">
        <div class="scores">
          <span>You: {scores.player}</span>
          <span>Opponent: {scores.opponent}</span>
        </div>
      </div>
    {:else if gameStatus === 'finished'}
      <p class="game-result">
        Game Over! Final Score: {scores.player} - {scores.opponent}
      </p>
    {/if}
  </div>

  {#if gameStatus === 'playing'}
    <div class="game-area">
      {#if myChoice && opponentChoice}
        <div class="choices-reveal">
          <div class="choice-display">
            <div class="your-choice">
              <h4>Your Choice</h4>
              <div class="choice-emoji">{choices.find(c => c.value === myChoice)?.emoji}</div>
              <p>{choices.find(c => c.value === myChoice)?.label}</p>
            </div>
            <div class="vs">VS</div>
            <div class="opponent-choice">
              <h4>Opponent</h4>
              <div class="choice-emoji">{choices.find(c => c.value === opponentChoice)?.emoji}</div>
              <p>{choices.find(c => c.value === opponentChoice)?.label}</p>
            </div>
          </div>
          {#if roundResult}
            <div class="round-result" class:win={roundResult === 'WIN'} class:lose={roundResult === 'LOSE'}>
              <h3>
                {roundResult === 'WIN' ? 'You Win!' : roundResult === 'LOSE' ? 'You Lose!' : 'Draw!'}
              </h3>
            </div>
          {/if}
        </div>
      {:else if myChoice}
        <div class="waiting-opponent">
          <p>You chose: <strong>{choices.find(c => c.value === myChoice)?.label}</strong></p>
          <p>Waiting for opponent...</p>
        </div>
      {:else}
        <div class="choice-buttons">
          <p>Make your choice:</p>
          <div class="buttons">
            {#each choices as choice}
              <button
                class="choice-btn"
                onclick={() => makeChoice(choice.value)}
                disabled={!wsConnected}
              >
                <div class="choice-emoji">{choice.emoji}</div>
                <span>{choice.label}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if !wsConnected}
    <p class="connection-status">⚠ Connecting to game server...</p>
  {/if}
</div>

<style>
  .rock-paper-scissors {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    min-height: 400px;
  }

  .game-info {
    text-align: center;
    margin-bottom: 2rem;
  }

  .game-info h3 {
    margin: 0 0 1rem 0;
    color: #333;
  }

  .game-status {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .scores {
    display: flex;
    gap: 2rem;
    justify-content: center;
    font-weight: bold;
  }

  .game-result {
    font-size: 1.2rem;
    font-weight: bold;
    color: #2e7d32;
  }

  .choice-buttons {
    text-align: center;
  }

  .buttons {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .choice-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    background: white;
    border: 2px solid #ddd;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    min-width: 100px;
  }

  .choice-btn:hover:not(:disabled) {
    border-color: #1976d2;
    transform: translateY(-2px);
  }

  .choice-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .choice-emoji {
    font-size: 3rem;
    margin-bottom: 0.5rem;
  }

  .choices-reveal {
    text-align: center;
  }

  .choice-display {
    display: flex;
    align-items: center;
    gap: 2rem;
    margin-bottom: 2rem;
  }

  .your-choice, .opponent-choice {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .vs {
    font-size: 1.5rem;
    font-weight: bold;
    color: #666;
  }

  .round-result {
    padding: 1rem;
    background: #e8f5e9;
    border-radius: 8px;
    color: #2e7d32;
  }

  .waiting-opponent {
    text-align: center;
    padding: 2rem;
  }

  .connection-status {
    margin-top: 1rem;
    color: #f57c00;
    font-style: italic;
  }
</style>
