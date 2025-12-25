<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import GameIframe from '$lib/components/GameIframe.svelte';
  import type { Room, Game, RoomMessage, CreateRoomMessageRequest } from '$lib/types';
  
  let roomId = $page.params.id;
  let room = $state<Room | null>(null);
  let game = $state<Game | null>(null);
  let messages = $state<RoomMessage[]>([]);
  let newMessage = $state('');
  let loading = $state(true);
  let ws: WebSocket | null = null;
  
  onMount(async () => {
    await loadRoomData();
    setupWebSocket();
    
    return () => {
      ws?.close();
    };
  });
  
  async function loadRoomData() {
    try {
      const roomData = await api.get<Room>(`/api/rooms/${roomId}`);
      room = roomData;
      
      const gameData = await api.get<Game>(`/api/games/${room.game_id}`);
      game = gameData;
      
      const messagesData = await api.get<RoomMessage[]>(`/api/rooms/${roomId}/messages?limit=50`);
      messages = messagesData.reverse();
    } catch (error) {
      console.error('Failed to load room data:', error);
    } finally {
      loading = false;
    }
  }
  
  function setupWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const apiHost = window.location.hostname.replace(/^[^.]+\./, 'api.');
    const wsUrl = `${protocol}//${apiHost}`;
    
    ws = new WebSocket(`${wsUrl}/ws/room/${roomId}`);
    
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      handleWebSocketMessage(message);
    };
  }
  
  function handleWebSocketMessage(message: any) {
    switch (message.type) {
      case 'chat_message':
        // Add to local messages (in real app, also save to DB)
        messages = [...messages, {
          id: Date.now().toString(),
          room_id: roomId,
          user_id: message.user_id,
          message: message.message,
          created_at: new Date().toISOString()
        }];
        break;
      case 'player_joined':
        console.log(`${message.username} joined the room`);
        break;
      case 'player_left':
        console.log(`Player ${message.user_id} left the room`);
        break;
    }
  }
  
  async function sendMessage() {
    if (!newMessage.trim()) return;
    
    try {
      const request: CreateRoomMessageRequest = {
        message: newMessage.trim()
      };
      
      await api.post(`/api/rooms/${roomId}/messages`, request, true);
      
      // Also send via WebSocket for real-time
      ws?.send(JSON.stringify({
        type: 'chat_message',
        message: newMessage.trim()
      }));
      
      newMessage = '';
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  }
  
  async function startGame() {
    try {
      await api.post(`/api/rooms/${roomId}/start`, {}, true);
      await loadRoomData();
    } catch (error) {
      console.error('Failed to start game:', error);
    }
  }
  
  async function leaveRoom() {
    try {
      await api.delete(`/api/rooms/${roomId}/leave`, true);
      window.location.href = '/rooms';
    } catch (error) {
      console.error('Failed to leave room:', error);
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
    <div class="room-header">
      <div class="room-info">
        <h1>{room.name}</h1>
        <p class="game-name">{game.name}</p>
        <div class="room-status">
          <span class="status status-{room.status}">{room.status}</span>
          <span class="players">{room.players.length}/{room.max_players} players</span>
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
          <GameIframe 
            gameCode={game.frontend_code}
            roomId={roomId}
            wsUrl={`${window.location.protocol === 'https:' ? 'https:' : 'http:'}//${window.location.hostname.replace(/^[^.]+\./, 'api.')}`}
          />
        {:else if room.status === 'waiting'}
          <div class="waiting-area">
            <h2>Waiting for game to start...</h2>
            <p>Players in room:</p>
            <ul class="player-list">
              {#each room.players as playerId}
                <li>Player {playerId.slice(-8)}</li>
              {/each}
            </ul>
          </div>
        {:else}
          <div class="game-finished">
            <h2>Game Finished</h2>
          </div>
        {/if}
      </div>
      
      <div class="chat-area">
        <div class="chat-header">
          <h3>Chat</h3>
        </div>
        <div class="chat-messages">
          {#each messages as message}
            <div class="message">
              <span class="username">Player {message.user_id.slice(-8)}:</span>
              <span class="text">{message.message}</span>
            </div>
          {/each}
        </div>
        <div class="chat-input">
          <input
            type="text"
            bind:value={newMessage}
            placeholder="Type a message..."
            onkeypress={handleKeyPress}
          />
          <button onclick={sendMessage}>Send</button>
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
    border-radius: 4px;
    font-size: 0.8rem;
    text-transform: capitalize;
  }
  
  .status-waiting { background: #e3f2fd; color: #1976d2; }
  .status-playing { background: #fff3e0; color: #f57c00; }
  .status-finished { background: #e8f5e8; color: #388e3c; }
  
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
    border-radius: 8px;
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
    border-radius: 4px;
  }
  
  .chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    min-width: 300px;
  }
  
  .chat-header {
    padding: 1rem;
    border-bottom: 1px solid #e0e0e0;
    background: #f9f9f9;
  }
  
  .chat-header h3 {
    margin: 0;
  }
  
  .chat-messages {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
    min-height: 0;
  }
  
  .message {
    margin-bottom: 0.5rem;
  }
  
  .username {
    font-weight: bold;
    color: #1976d2;
  }
  
  .text {
    margin-left: 0.5rem;
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
    border-radius: 4px 0 0 4px;
  }
  
  .chat-input button {
    padding: 0.5rem 1rem;
    background: #1976d2;
    color: white;
    border: none;
    border-radius: 0 4px 4px 0;
    cursor: pointer;
  }
  
  .btn-primary, .btn-secondary {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
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
