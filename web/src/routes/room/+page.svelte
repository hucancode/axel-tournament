<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { roomService } from '$services/rooms';
  import { gameService } from '$services/games';
  import { RoomSocket } from '$services/roomSocket';
  import { createGame } from '$lib/games/registry';
  import type { BasePixiGame } from '$lib/games/BasePixiGame';
  import { Alert } from "$components";
  import type { Room, Game } from '$lib/models';

  const roomId = $derived(page.url.searchParams.get('id') || '');
  let room = $state<Room | null>(null);
  let game = $state<Game | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let roomSocket: RoomSocket | null = null;
  let wsConnected = $state(false);
  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  let pixiGame: BasePixiGame | null = null;
  let chatMessages = $state<Array<{userId: string, username: string, message: string}>>([]);
  let chatInput = $state('');

  onMount(() => {
    if (!roomId) {
      goto('/rooms');
      return;
    }
    loadRoomData().then(() => {
      setupWebSocket();
    });

    return () => {
      pixiGame?.destroy();
      roomSocket?.disconnect();
    };
  });

  async function loadRoomData() {
    try {
      loading = true;
      error = null;

      const roomData = await roomService.get(roomId);
      room = roomData;

      const gameData = await gameService.get(room.game_id);
      game = gameData;

      // Initialize PixiJS game if room is playing
      if (room.status === 'playing' && canvas) {
        initializeGame();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load room data';
      console.error('Failed to load room data:', err);
    } finally {
      loading = false;
    }
  }

  function initializeGame() {
    if (!room || !canvas || !roomSocket) return;

    pixiGame?.destroy();
    const sendMove = (message: string) => {
      roomSocket!.sendMove(message);
    };
    pixiGame = createGame(room.game_id, canvas, sendMove, wsConnected);
  }

  async function setupWebSocket() {
    if (!game || !room) {
      console.error('Cannot setup WebSocket: game or room not loaded');
      return;
    }

    try {
      roomSocket = new RoomSocket(room.game_id, roomId);

      // Set up event handlers
      roomSocket.on('authenticated', (userId) => {
        wsConnected = true;
        console.log('Authenticated as:', userId);
      });

      roomSocket.on('reconnect', (userId) => {
        wsConnected = true;
        console.log('Reconnected as:', userId);
      });

      roomSocket.on('auth_failed', (reason) => {
        error = `Authentication failed: ${reason}`;
        wsConnected = false;
      });

      roomSocket.on('player_joined', (data) => {
        console.log('Player joined:', data);
        // Refresh room data
        loadRoomData();
      });

      roomSocket.on('player_left', (data) => {
        console.log('Player left:', data);
        // Refresh room data
        loadRoomData();
      });

      roomSocket.on('host_changed', (data) => {
        console.log('Host changed:', data);
        // Refresh room data
        loadRoomData();
      });

      roomSocket.on('game_started', async () => {
        console.log('Game started, game type:', room?.game_id);
        // Update room status to trigger canvas render
        if (room) {
          room.status = 'playing';
        }
        // Wait for Svelte to update DOM and render canvas
        await tick();
        // Initialize game so it's ready for START message
        if (canvas && room && !pixiGame) {
          console.log('Initializing game for:', room.game_id);
          initializeGame();
        } else {
          console.log('Skipping init - canvas:', !!canvas, 'room:', !!room, 'pixiGame already exists:', !!pixiGame);
        }
      });

      roomSocket.on('chat', (data: any) => {
        if (typeof data === 'object' && data.userId && data.username && data.message) {
          chatMessages = [...chatMessages, data as {userId: string, username: string, message: string}];
        }
      });

      roomSocket.on('error', (message) => {
        error = message;
      });

      roomSocket.on('disconnect', () => {
        wsConnected = false;
      });

      // Game-specific events - forward to PixiJS game
      roomSocket.on('game_start', (data) => {
        console.log('Game START:', data, 'pixiGame exists:', !!pixiGame);
        if (pixiGame) {
          const message = data ? `START ${data}` : 'START';
          console.log('Forwarding to game:', message);
          pixiGame.handleMessage(message);
        } else {
          console.error('pixiGame not initialized yet when START received');
        }
      });

      roomSocket.on('board_update', (boardData) => {
        console.log('Board update:', boardData);
        if (pixiGame) {
          pixiGame.handleMessage(`BOARD ${boardData}`);
        }
      });

      roomSocket.on('your_turn', () => {
        console.log('Your turn!');
        if (pixiGame) {
          pixiGame.handleMessage('YOUR_TURN');
        }
      });

      roomSocket.on('turn_update', (data) => {
        console.log('Turn update:', data);
        if (pixiGame) {
          pixiGame.handleMessage(`TURN ${data}`);
        }
      });

      roomSocket.on('score_update', (data) => {
        console.log('Score update:', data);
        if (pixiGame) {
          pixiGame.handleMessage(`SCORE ${data}`);
        }
      });

      roomSocket.on('game_end', () => {
        console.log('Game ended');
        if (pixiGame) {
          pixiGame.handleMessage('END');
        }
      });
      // Handle generic messages (like ROUND, etc.) that fall through
      roomSocket.on('message', (data) => {
        if (pixiGame) {
          pixiGame.handleMessage(data);
        }
      });
      // Connect and authenticate
      await roomSocket.connect();

      const token = localStorage.getItem('auth_token');
      if (!token) {
        error = 'No authentication token found';
        return;
      }

      await roomSocket.auth(token);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to setup WebSocket';
      console.error('Failed to setup WebSocket:', err);
    }
  }

  async function startGame() {
    if (!roomSocket || !roomSocket.isAuthenticated()) {
      error = 'WebSocket not connected';
      return;
    }

    try {
      roomSocket.startGame();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to start game';
      console.error('Failed to start game:', err);
    }
  }

  function leaveRoom() {
    if (roomSocket?.isAuthenticated()) {
      roomSocket.leave();
    }
    goto('/rooms');
  }

  function sendChat() {
    if (!roomSocket || !roomSocket.isAuthenticated() || !chatInput.trim()) {
      return;
    }

    try {
      roomSocket.chat(chatInput);
      chatInput = '';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to send chat message';
      console.error('Failed to send chat:', err);
    }
  }
</script>

{#if loading}
  <div class="loading">Loading room...</div>
{:else if !room || !game}
  <div class="error">Room not found</div>
{:else}
  <div class="room-container">
    {#if error}
      <Alert type="error" message={error} onclose={() => error = null} />
    {/if}

    <div class="room-header">
      <div class="room-info">
        <h1>{room.name}</h1>
        <p class="game-name">{game.name}</p>
        <div class="room-status">
          <span class="status status-{room.status}">{room.status}</span>
          <span class="players">👥 {room.players.length}/{room.max_players}</span>
          <span class="ws-status" class:connected={wsConnected}>
            {wsConnected ? '🟢 Live' : '🔴 Offline'}
          </span>
        </div>
      </div>
      <div class="room-actions">
        {#if room.status === 'waiting'}
          <button data-variant="primary" onclick={startGame}>Start Game</button>
        {/if}
        <button data-variant="secondary" onclick={leaveRoom}>Leave Room</button>
      </div>
    </div>

    <div class="room-content">
      <div class="game-area">
        {#if room.status === 'playing'}
          <div class="playing-area">
            <canvas bind:this={canvas}></canvas>
          </div>
        {:else if room.status === 'waiting'}
          <div class="waiting-area">
            <h2>Waiting for game to start...</h2>
            <p>Players in room:</p>
            <ul class="player-list">
              {#each room.players as playerId}
                <li>👤 Player {playerId.slice(-8)}</li>
              {/each}
            </ul>
            {#if room.players.length < 2}
              <p class="hint">Need at least 2 players to start</p>
            {/if}
          </div>
        {:else}
          <div class="game-finished">
            <h2>Game Finished</h2>
          </div>
        {/if}
      </div>

      <!-- Chat Panel -->
      {#if wsConnected}
        <div class="chat-panel">
          <div class="chat-header">
            <h3>Chat</h3>
          </div>
          <div class="chat-messages">
            {#each chatMessages as msg}
              <div class="chat-message">
                <strong>{msg.username}:</strong> {msg.message}
              </div>
            {/each}
          </div>
          <div class="chat-input">
            <input
              type="text"
              bind:value={chatInput}
              placeholder="Type a message..."
              onkeydown={(e) => e.key === 'Enter' && sendChat()}
            />
            <button onclick={sendChat}>Send</button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .room-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 1rem;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .room-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border-light);
    margin-bottom: 1rem;
  }

  .room-info h1 {
    margin: 0 0 0.5rem 0;
  }

  .game-name {
    color: var(--color-fg-dim);
    margin: 0 0 0.5rem 0;
  }

  .room-status {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .status {
    padding: 0.25rem 0.5rem;
    font-size: 0.8rem;
    text-transform: capitalize;
  }

  .status-waiting { background: var(--color-info); color: var(--color-bg); }
  .status-playing { background: var(--color-warning); color: var(--color-bg); }
  .status-finished { background: var(--color-success); color: var(--color-bg); }

  .ws-status {
    padding: 0.25rem 0.5rem;
    font-size: 0.8rem;
    background: var(--color-error);
    color: var(--color-bg);
  }

  .ws-status.connected {
    background: var(--color-success);
    color: var(--color-bg);
  }

  .hint {
    color: var(--color-warning);
    font-style: italic;
    margin-top: 1rem;
  }

  .room-actions {
    display: flex;
    gap: 1rem;
  }

  .room-content {
    display: flex;
    flex: 1;
    gap: 1rem;
    min-height: 0;
  }

  .game-area {
    flex: 2;
    min-height: 0;
  }

  .chat-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    max-width: 300px;
  }

  .chat-header {
    padding: 1rem;
    border-bottom: 1px solid #e0e0e0;
    background: #f5f5f5;
    border-radius: 8px 8px 0 0;
  }

  .chat-header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .chat-messages {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
    min-height: 200px;
    max-height: 400px;
  }

  .chat-message {
    margin-bottom: 0.5rem;
    padding: 0.5rem;
    background: #f9f9f9;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .chat-input {
    display: flex;
    padding: 1rem;
    border-top: 1px solid #e0e0e0;
    gap: 0.5rem;
  }

  .chat-input input {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
  }

  .chat-input button {
    padding: 0.5rem 1rem;
    background: #1976d2;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .playing-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: #f9f9f9;
    padding: 2rem;
  }

  canvas {
    border: 1px solid #ddd;
    border-radius: 8px;
  }

  .waiting-area, .game-finished {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: #f9f9f9;
    padding: 2rem;
  }

  .player-list {
    list-style: none;
    padding: 0;
  }

  .player-list li {
    padding: 0.5rem;
    background: white;
    margin: 0.25rem 0;
  }

  .loading, .error {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    font-size: 1.2rem;
    color: #666;
  }
</style>
