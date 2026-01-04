<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { roomService } from '$lib/services/rooms';
  import { gameService } from '$lib/services/games';
  import { RoomSocket } from '$lib/services/roomSocket';
  import { createGame } from '$lib/games/registry';
  import type { BasePixiGame } from '$lib/games/BasePixiGame';
  import Alert from '$lib/components/Alert.svelte';
  import type { Room, Game } from '$lib/types';

  const roomId = page.params.id!;
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
    if (!room || !canvas) return;

    pixiGame?.destroy();
    pixiGame = createGame(room.game_id, canvas, null, wsConnected);
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

      roomSocket.on('game_started', () => {
        console.log('Game started');
        // Refresh room data and initialize game
        loadRoomData().then(() => {
          if (canvas) {
            initializeGame();
          }
        });
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

      // Game-specific events
      roomSocket.on('board_update', (boardData) => {
        console.log('Board update:', boardData);
        // Forward to PixiJS game if available
        if (pixiGame) {
          // pixiGame.handleMessage(`BOARD ${boardData}`);
        }
      });

      roomSocket.on('your_turn', () => {
        console.log('Your turn!');
        // Forward to PixiJS game if available
        if (pixiGame) {
          // pixiGame.handleMessage('YOUR_TURN');
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

  async function leaveRoom() {
    try {
      if (roomSocket && roomSocket.isAuthenticated()) {
        roomSocket.leave();
      }
      await roomService.leave(roomId);
      goto('/rooms');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to leave room';
      console.error('Failed to leave room:', err);
    }
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
          <button class="btn-primary" onclick={startGame}>Start Game</button>
        {/if}
        <button class="btn-secondary" onclick={leaveRoom}>Leave Room</button>
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
    border-bottom: 1px solid #e0e0e0;
    margin-bottom: 1rem;
  }

  .room-info h1 {
    margin: 0 0 0.5rem 0;
  }

  .game-name {
    color: #666;
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

  .status-waiting { background: #e3f2fd; color: #1976d2; }
  .status-playing { background: #fff3e0; color: #f57c00; }
  .status-finished { background: #e8f5e8; color: #388e3c; }

  .ws-status {
    padding: 0.25rem 0.5rem;
    font-size: 0.8rem;
    background: #ffebee;
    color: #c62828;
  }

  .ws-status.connected {
    background: #e8f5e9;
    color: #2e7d32;
  }

  .hint {
    color: #f57c00;
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


  .btn-primary, .btn-secondary {
    padding: 0.5rem 1rem;
    cursor: pointer;
  }

  .btn-primary {
    background: #1976d2;
    color: white;
  }

  .btn-secondary {
    background: #f5f5f5;
    color: #333;
    border: 1px solid #ddd;
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
