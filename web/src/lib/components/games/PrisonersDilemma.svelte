<script lang="ts">
  interface Props {
    ws: WebSocket | null;
    wsConnected: boolean;
  }

  let { ws, wsConnected }: Props = $props();

  let myChoice = $state<'COOPERATE' | 'DEFECT' | null>(null);
  let opponentChoice = $state<'COOPERATE' | 'DEFECT' | null>(null);
  let gameStatus = $state<'waiting' | 'playing' | 'finished'>('waiting');
  let currentRound = $state(1);
  let totalRounds = $state(10);
  let scores = $state<{player: number, opponent: number}>({player: 0, opponent: 0});
  let history = $state<Array<{round: number, player: string, opponent: string, playerScore: number, opponentScore: number}>>([]);

  const choices = [
    { value: 'COOPERATE', emoji: '🤝', label: 'Cooperate', description: 'Work together' },
    { value: 'DEFECT', emoji: '⚔️', label: 'Defect', description: 'Betray opponent' }
  ] as const;

  function makeChoice(choice: 'COOPERATE' | 'DEFECT') {
    if (!wsConnected || !ws || gameStatus !== 'playing' || myChoice !== null) {
      return;
    }

    myChoice = choice;
    // Send move in text protocol format: "COOPERATE" or "DEFECT"
    ws.send(choice);
  }

  function handleTextMessage(text: string) {
    const trimmed = text.trim();
    const parts = trimmed.split(/\s+/);

    if (parts.length === 0) return;

    switch (parts[0]) {
      case 'START':
        // Format: "START rounds" (e.g., "START 10")
        if (parts.length === 2) {
          totalRounds = parseInt(parts[1]);
        }
        gameStatus = 'playing';
        currentRound = 1;
        break;

      case 'ROUND':
        // Format: "ROUND n"
        if (parts.length === 2) {
          currentRound = parseInt(parts[1]);
          // Reset choices for new round
          myChoice = null;
          opponentChoice = null;
        }
        break;

      case 'RESULT':
        // Format: "RESULT your_choice opp_choice your_points opp_points"
        // Example: "RESULT COOPERATE DEFECT 0 5"
        if (parts.length === 5) {
          const yourChoice = parts[1] as 'COOPERATE' | 'DEFECT';
          const oppChoice = parts[2] as 'COOPERATE' | 'DEFECT';
          const yourPoints = parseInt(parts[3]);
          const oppPoints = parseInt(parts[4]);

          myChoice = yourChoice;
          opponentChoice = oppChoice;

          // Add to history
          history = [...history, {
            round: currentRound,
            player: yourChoice,
            opponent: oppChoice,
            playerScore: yourPoints,
            opponentScore: oppPoints
          }];

          // Reset choices after 2 seconds
          setTimeout(() => {
            myChoice = null;
            opponentChoice = null;
          }, 2000);
        }
        break;

      case 'SCORE':
        // Format: "SCORE total_you total_opp"
        if (parts.length === 3) {
          scores = {
            player: parseInt(parts[1]),
            opponent: parseInt(parts[2])
          };
        }
        break;

      case 'END':
        // Format: "END total_you total_opp"
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

<div class="prisoners-dilemma">
  <div class="game-info">
    <h3>Prisoner's Dilemma</h3>
    {#if gameStatus === 'waiting'}
      <p>Waiting for players...</p>
    {:else if gameStatus === 'playing'}
      <div class="game-status">
        <p>Round: <strong>{currentRound}/{totalRounds}</strong></p>
        <div class="scores">
          <span>Your Score: {scores.player}</span>
          <span>Opponent Score: {scores.opponent}</span>
        </div>
      </div>
    {:else if gameStatus === 'finished'}
      <p class="game-result">
        Game Over! Final Score: {scores.player} - {scores.opponent}
        {scores.player > scores.opponent ? ' - You Win!' : scores.player < scores.opponent ? ' - You Lose!' : ' - Draw!'}
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
          <div class="round-outcome">
            <p>Round {currentRound} complete!</p>
          </div>
        </div>
      {:else if myChoice}
        <div class="waiting-opponent">
          <p>You chose: <strong>{choices.find(c => c.value === myChoice)?.label}</strong></p>
          <p>Waiting for opponent...</p>
        </div>
      {:else}
        <div class="choice-buttons">
          <div class="dilemma-explanation">
            <h4>The Dilemma:</h4>
            <ul>
              <li><strong>Both Cooperate:</strong> Both get 3 points</li>
              <li><strong>Both Defect:</strong> Both get 1 point</li>
              <li><strong>One Defects, One Cooperates:</strong> Defector gets 5 points, Cooperator gets 0</li>
            </ul>
          </div>
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
                <small>{choice.description}</small>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if history.length > 0}
    <div class="history">
      <h4>Game History</h4>
      <div class="history-table">
        <div class="history-header">
          <span>Round</span>
          <span>You</span>
          <span>Opponent</span>
          <span>Your Points</span>
          <span>Opp Points</span>
        </div>
        {#each history.slice(-5) as round}
          <div class="history-row">
            <span>{round.round}</span>
            <span>{round.player === 'COOPERATE' ? '🤝' : '⚔️'}</span>
            <span>{round.opponent === 'COOPERATE' ? '🤝' : '⚔️'}</span>
            <span>{round.playerScore}</span>
            <span>{round.opponentScore}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if !wsConnected}
    <p class="connection-status">⚠ Connecting to game server...</p>
  {/if}
</div>

<style>
  .prisoners-dilemma {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    min-height: 500px;
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

  .dilemma-explanation {
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
    text-align: left;
  }

  .dilemma-explanation ul {
    margin: 0.5rem 0;
    padding-left: 1.5rem;
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
    min-width: 120px;
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

  .choice-btn small {
    color: #666;
    font-size: 0.8rem;
    margin-top: 0.25rem;
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

  .round-outcome {
    padding: 1rem;
    background: #e8f5e9;
    border-radius: 8px;
    color: #2e7d32;
  }

  .waiting-opponent {
    text-align: center;
    padding: 2rem;
  }

  .history {
    margin-top: 2rem;
    width: 100%;
    max-width: 500px;
  }

  .history h4 {
    margin-bottom: 1rem;
    text-align: center;
  }

  .history-table {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .history-header, .history-row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 1fr 1fr;
    gap: 0.5rem;
    padding: 0.5rem;
    text-align: center;
  }

  .history-header {
    background: #f5f5f5;
    font-weight: bold;
    border-radius: 4px;
  }

  .history-row {
    background: white;
    border: 1px solid #eee;
    border-radius: 4px;
  }

  .connection-status {
    margin-top: 1rem;
    color: #f57c00;
    font-style: italic;
  }
</style>
