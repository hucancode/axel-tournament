<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { roomService } from '$lib/services/rooms';
  import { gameService } from '$lib/services/games';
  import GameIframe from '$lib/components/GameIframe.svelte';
  import Alert from '$lib/components/Alert.svelte';
  import type { Room, Game, RoomMessage, CreateRoomMessageRequest } from '$lib/types';

  const roomId = page.params.id!;
  let room = $state<Room | null>(null);
  let game = $state<Game | null>(null);
  let messages = $state<RoomMessage[]>([]);
  let newMessage = $state('');
  let loading = $state(true);
  let error = $state<string | null>(null);
  let ws: WebSocket | null = null;
  let wsConnected = $state(false);
  let chatContainer = $state<HTMLDivElement | null>(null);

  // WebSocket URL for interactive-judge (constant for the session)
  const gameWsUrl = window.location.host.includes('localhost')
    ? 'http://localhost:8081'
    : `${window.location.protocol}//${window.location.host}`;

  onMount(() => {
    if (!roomId) {
      goto('/rooms');
      return;
    }
    loadRoomData();
    setupWebSocket();

    return () => {
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

      const messagesData = await roomService.getMessages(roomId, 50);
      messages = messagesData;

      // Scroll to bottom after messages load
      setTimeout(scrollToBottom, 100);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load room data';
      console.error('Failed to load room data:', err);
    } finally {
      loading = false;
    }
  }

  function setupWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';

    // In development, connect to interactive-judge on port 8081
    // In production, use the same host (ingress routes /ws to interactive-judge)
    const host = window.location.host.includes('localhost')
      ? 'localhost:8081'
      : window.location.host;

    const wsUrl = `${protocol}//${host}/ws/room/${roomId}`;

    console.log('Connecting to WebSocket:', wsUrl);
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      wsConnected = true;
      console.log('WebSocket connected to interactive-judge');
    };

    ws.onclose = () => {
      wsConnected = false;
      console.log('WebSocket disconnected');
    };

    ws.onerror = (err) => {
      wsConnected = false;
      console.error('WebSocket error:', err);
    };

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      handleWebSocketMessage(message);
    };
  }

  function handleWebSocketMessage(data: any) {
    switch (data.type) {
      case 'chat_message':
        // Interactive-judge sends: { type, user_id, username, message }
        // Convert to RoomMessage format for display
        const chatMessage: RoomMessage = {
          id: Date.now().toString(),
          room_id: roomId,
          user_id: data.user_id,
          message: data.message,
          created_at: new Date().toISOString()
        };
        messages = [...messages, chatMessage];
        scrollToBottom();
        break;
      case 'player_joined':
        error = null;
        console.log(`${data.username} joined the room`);
        // Only reload if game hasn't started yet
        if (room?.status === 'waiting') {
          loadRoomData();
        }
        break;
      case 'player_left':
        console.log(`Player ${data.user_id} left the room`);
        // Only reload if game hasn't started yet
        if (room?.status === 'waiting') {
          loadRoomData();
        }
        break;
      case 'game_started':
        loadRoomData();
        break;
    }
  }

  function scrollToBottom() {
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  }

  async function sendMessage() {
    if (!newMessage.trim() || !ws || ws.readyState !== WebSocket.OPEN) return;

    try {
      error = null;

      // Send via WebSocket to interactive-judge for real-time broadcast
      const wsMessage = {
        type: 'chat_message',
        message: newMessage.trim()
      };
      ws.send(JSON.stringify(wsMessage));

      // Also save to database via REST API (for message persistence)
      const request: CreateRoomMessageRequest = {
        message: newMessage.trim()
      };
      await roomService.sendMessage(roomId, request);

      newMessage = '';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to send message';
      console.error('Failed to send message:', err);
    }
  }

  async function startGame() {
    try {
      error = null;
      await roomService.start(roomId);
      await loadRoomData();
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

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
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
        {#if room.status === 'playing' && game.frontend_code}
          {#key roomId}
            <GameIframe
              gameCode={game.frontend_code || ''}
              roomId={roomId}
              wsUrl={gameWsUrl}
            />
          {/key}
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

      <div class="chat-area">
        <div class="chat-header">
          <h3>💬 Chat</h3>
          {#if !wsConnected}
            <span class="chat-warning">Real-time disabled</span>
          {/if}
        </div>
        <div class="chat-messages" bind:this={chatContainer}>
          {#if messages.length === 0}
            <div class="no-messages">No messages yet. Start the conversation!</div>
          {:else}
            {#each messages as message}
              <div class="message">
                <div class="message-header">
                  <span class="username">Player {message.user_id.slice(-8)}</span>
                  <span class="timestamp">{new Date(message.created_at).toLocaleTimeString()}</span>
                </div>
                <div class="message-text">{message.message}</div>
              </div>
            {/each}
          {/if}
        </div>
        <div class="chat-input">
          <input
            type="text"
            bind:value={newMessage}
            placeholder="Type a message..."
            onkeypress={handleKeyPress}
            maxlength="500"
          />
          <button onclick={sendMessage} disabled={!newMessage.trim()}>Send</button>
        </div>
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

  .chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    border: 1px solid #e0e0e0;
    min-width: 300px;
  }

  .chat-header {
    padding: 1rem;
    border-bottom: 1px solid #e0e0e0;
    background: #f9f9f9;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .chat-header h3 {
    margin: 0;
  }

  .chat-warning {
    font-size: 0.75rem;
    color: #f57c00;
    background: #fff3e0;
    padding: 0.25rem 0.5rem;
  }

  .chat-messages {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
    min-height: 0;
    background: #fafafa;
  }

  .no-messages {
    text-align: center;
    color: #999;
    padding: 2rem;
    font-style: italic;
  }

  .message {
    margin-bottom: 1rem;
    padding: 0.75rem;
    background: white;
    box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .username {
    font-weight: 600;
    color: #1976d2;
    font-size: 0.9rem;
  }

  .timestamp {
    font-size: 0.75rem;
    color: #999;
  }

  .message-text {
    color: #333;
    line-height: 1.4;
    word-wrap: break-word;
  }

  .chat-input {
    display: flex;
    padding: 1rem;
    border-top: 1px solid #e0e0e0;
  }

  .chat-input input {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #ddd;
  }

  .chat-input button {
    padding: 0.5rem 1rem;
    background: #1976d2;
    color: white;
    cursor: pointer;
    transition: all 0.2s;
  }

  .chat-input button:hover:not(:disabled) {
    background: #1565c0;
  }

  .chat-input button:disabled {
    background: #ccc;
    cursor: not-allowed;
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
