<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { gameService } from '$services/games';
  import { RoomSocket, type RoomEvent, type RoomError } from '$services/roomSocket';
  import { createGame } from '$lib/games/registry';
  import type { BasePixiGame } from '$lib/games/BasePixiGame';
  import { Alert } from '$components';
  import type { Game } from '$lib/models';

  // URL: /room?id=<room_id>&game=<game_id>
  type Phase = 'lobby' | 'playing' | 'finished';

  const roomId = $derived(page.url.searchParams.get('id') || '');
  const gameId = $derived(page.url.searchParams.get('game') || '');

  let gameInfo = $state<Game | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let roomSocket: RoomSocket | null = null;
  let wsConnected = $state(false);
  let players = $state<string[]>([]);
  let host = $state<string | null>(null);
  let phase = $state<Phase>('lobby');
  let myPlayerId = $state<string | null>(null);

  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  let pixiGame: BasePixiGame | null = null;

  let chatMessages = $state<{ userId: string; message: string }[]>([]);
  let chatInput = $state('');

  onMount(() => {
    if (!roomId || !gameId) {
      goto('/rooms');
      return;
    }
    loadGame().then(() => setupSocket());
    return () => {
      pixiGame?.destroy();
      roomSocket?.disconnect();
    };
  });

  async function loadGame() {
    try {
      loading = true;
      error = null;
      gameInfo = await gameService.get(gameId);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load game metadata';
      console.error('Failed to load game:', err);
    } finally {
      loading = false;
    }
  }

  function initializeGame() {
    if (!canvas || !roomSocket) return;
    pixiGame?.destroy();
    pixiGame = createGame(
      gameId,
      canvas,
      (kind, payload) => roomSocket!.act(kind, payload),
      wsConnected,
    );
  }

  /** Fold the room-kernel envelope (PLAYER_JOINED/LEFT/HOST/START/END/CHAT)
   *  shared by every game. Per-game events flow on to the pixi module. */
  async function applyEvent(e: RoomEvent) {
    switch (e.kind) {
      case 'PLAYER_JOINED':
        if (!players.includes(e.payload)) players = [...players, e.payload];
        if (!host) host = e.payload;
        break;
      case 'PLAYER_LEFT':
        players = players.filter((p) => p !== e.payload);
        if (host === e.payload) host = players[0] ?? null;
        break;
      case 'HOST_CHANGED':
        host = e.payload;
        break;
      case 'GAME_STARTED':
        phase = 'playing';
        await tick();
        if (canvas && !pixiGame) initializeGame();
        break;
      case 'GAME_END':
      case 'WINNER':
      case 'DRAW':
        phase = 'finished';
        break;
      case 'CHAT': {
        const sp = e.payload.indexOf(' ');
        const userId = sp < 0 ? e.payload : e.payload.slice(0, sp);
        const message = sp < 0 ? '' : e.payload.slice(sp + 1);
        chatMessages = [...chatMessages, { userId, message }];
        break;
      }
    }
    pixiGame?.handleEvent(e.kind, e.payload, {
      myIndex: myPlayerId ? players.indexOf(myPlayerId) : -1,
      players,
    });
  }

  async function setupSocket() {
    const token = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;
    if (!token) {
      error = 'No authentication token found';
      return;
    }

    roomSocket = new RoomSocket(gameId, roomId);
    roomSocket.setHandlers({
      onConnected: (pid) => {
        wsConnected = true;
        myPlayerId = pid;
      },
      onDisconnect: () => {
        wsConnected = false;
      },
      onError: (e: RoomError) => {
        error = `${e.code}: ${e.msg}`;
      },
      onEvent: (e) => {
        void applyEvent(e);
      },
    });

    try {
      await roomSocket.connect(token);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to connect';
      console.error('Connect failed:', err);
    }
  }

  function joinRoom() {
    roomSocket?.act('JOIN');
  }

  function startGame() {
    if (!roomSocket?.isConnected()) {
      error = 'Not connected';
      return;
    }
    roomSocket.act('START');
  }

  function leaveRoom() {
    roomSocket?.act('LEAVE');
    roomSocket?.disconnect();
    goto('/rooms');
  }

  function sendChat() {
    if (!roomSocket?.isConnected() || !chatInput.trim()) return;
    roomSocket.act('CHAT', chatInput);
    chatInput = '';
  }

  const isHost = $derived(myPlayerId !== null && host === myPlayerId);
  const hasJoined = $derived(myPlayerId !== null && players.includes(myPlayerId));
  const statusClass = $derived(
    phase === 'lobby' ? 'status-waiting' : phase === 'playing' ? 'status-playing' : 'status-finished',
  );
  const statusLabel = $derived(
    phase === 'lobby' ? 'waiting' : phase === 'playing' ? 'playing' : 'finished',
  );
</script>

{#if loading}
  <div class="loading">Loading room...</div>
{:else if !gameInfo}
  <div class="error">Game not found</div>
{:else}
  <div class="room-container">
    {#if error}
      <Alert type="error" message={error} onclose={() => (error = null)} />
    {/if}

    <div class="room-header">
      <div class="room-info">
        <h1>{roomId}</h1>
        <p class="game-name">{gameInfo.name}</p>
        <div class="room-status">
          <span class="status {statusClass}">{statusLabel}</span>
          <span class="players">👥 {players.length}/2</span>
          <span class="ws-status" class:connected={wsConnected}>
            {wsConnected ? '🟢 Live' : '🔴 Offline'}
          </span>
        </div>
      </div>
      <div class="room-actions">
        {#if phase === 'lobby' && wsConnected && !hasJoined}
          <button data-variant="primary" onclick={joinRoom}>Join Room</button>
        {/if}
        {#if phase === 'lobby' && isHost && players.length >= 2}
          <button data-variant="primary" onclick={startGame}>Start Game</button>
        {/if}
        <button data-variant="secondary" onclick={leaveRoom}>Leave Room</button>
      </div>
    </div>

    <div class="room-content">
      <div class="game-area">
        {#if phase === 'playing'}
          <div class="playing-area">
            <canvas bind:this={canvas}></canvas>
          </div>
        {:else if phase === 'lobby'}
          <div class="waiting-area">
            <h2>Waiting for game to start...</h2>
            <p>Players in room:</p>
            <ul class="player-list">
              {#each players as pid}
                <li>👤 {pid.slice(-8)}{pid === host ? ' (host)' : ''}</li>
              {/each}
            </ul>
            {#if players.length < 2}
              <p class="hint">Need at least 2 players to start</p>
            {/if}
          </div>
        {:else}
          <div class="game-finished">
            <h2>Game Finished</h2>
          </div>
        {/if}
      </div>

      {#if wsConnected}
        <div class="chat-panel">
          <div class="chat-header"><h3>Chat</h3></div>
          <div class="chat-messages">
            {#each chatMessages as msg}
              <div class="chat-message">
                <strong>{msg.userId.slice(-8)}:</strong> {msg.message}
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
