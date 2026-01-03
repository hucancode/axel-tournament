<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { roomService } from '$lib/services/rooms';
  import { gameService } from '$lib/services/games';
  import { createGame } from '$lib/games/registry';
  import type { BasePixiGame } from '$lib/games/BasePixiGame';
  import Alert from '$lib/components/Alert.svelte';
  import type { Room, Game } from '$lib/types';

  const roomId = page.params.id!;
  let room = $state<Room | null>(null);
  let game = $state<Game | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let ws: WebSocket | null = null;
  let wsConnected = $state(false);
  let canvas: HTMLCanvasElement;
  let pixiGame: BasePixiGame | null = null;

  const roomId = page.params.id!;
  let room = $state<Room | null>(null);
  let game = $state<Game | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let ws: WebSocket | null = null;
  let wsConnected = $state(false);

  function getGameWebSocketPath(gameId: string): string {
    // Map game IDs to WebSocket paths
    const gamePathMap: Record<string, string> = {
      'tic-tac-toe': 'tic-tac-toe',
      'rock-paper-scissors': 'rock-paper-scissors', 
      'prisoners-dilemma': 'prisoners-dilemma'
    };
    
    return gamePathMap[gameId] || 'tic-tac-toe'; // fallback to tic-tac-toe
  }

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
      ws?.close();
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
    pixiGame = createGame(room.game_id, canvas, ws, wsConnected);
  }

  function setupWebSocket() {
    if (!game || !room) {
      console.error('Cannot setup WebSocket: game or room not loaded');
      return;
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const gamePath = getGameWebSocketPath(room.game_id);

    // Connect to the unified judge server with game-specific path
    const host = window.location.host.includes('localhost')
      ? 'localhost:8081'
      : window.location.host; // In production, ingress routes /room to judge:8081

    const wsUrl = `${protocol}//${host}/room/${gamePath}/${roomId}`;

    console.log(`Connecting to ${game.name} game server:`, wsUrl);
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      wsConnected = true;
      console.log(`WebSocket connected to ${game?.name} game server`);

      // Send authentication message - simple text-based protocol
      const userId = 'user_' + Math.random().toString(36).substr(2, 9); // Generate temp user ID
      ws!.send(userId);
    };

    ws.onclose = () => {
      wsConnected = false;
      console.log('WebSocket disconnected');
    };

    ws.onerror = (err) => {
      wsConnected = false;
      console.error('WebSocket error:', err);
    };

    // Game components will handle their own messages
    // No need to handle messages here unless we add chat or other room features
  }


  async function startGame() {
    try {
      error = null;
      await roomService.start(roomId);
      console.log('Game started via API');

      await loadRoomData();
      
      // Initialize game after status change
      if (canvas) {
        initializeGame();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to start game';
      console.error('Failed to start game:', err);
    }
  }

  async function leaveRoom() {
    try {
      error = null;
      await roomService.leave(roomId);
      goto('/rooms');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to leave room';
      console.error('Failed to leave room:', err);
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
