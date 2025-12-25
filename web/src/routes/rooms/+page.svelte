<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Room, Game, CreateRoomRequest } from '$lib/types';
  
  let rooms = $state<Room[]>([]);
  let games = $state<Game[]>([]);
  let loading = $state(true);
  let showCreateModal = $state(false);
  let selectedGameId = $state('');
  let roomName = $state('');
  let maxPlayers = $state(4);
  
  onMount(async () => {
    await loadData();
  });
  
  async function loadData() {
    try {
      const [roomsData, gamesData] = await Promise.all([
        api.get<Room[]>('/api/rooms'),
        api.get<Game[]>('/api/games')
      ]);
      rooms = roomsData;
      games = gamesData.filter(g => g.game_type === 'interactive');
    } catch (error) {
      console.error('Failed to load data:', error);
    } finally {
      loading = false;
    }
  }
  
  async function createRoom() {
    if (!selectedGameId || !roomName.trim()) return;
    
    try {
      const request: CreateRoomRequest = {
        game_id: selectedGameId,
        name: roomName.trim(),
        max_players: maxPlayers
      };
      
      await api.post<Room>('/api/rooms', request, true);
      await loadData();
      closeCreateModal();
    } catch (error) {
      console.error('Failed to create room:', error);
    }
  }
  
  async function joinRoom(roomId: string) {
    try {
      await api.post(`/api/rooms/${roomId}/join`, {}, true);
      // Navigate to room page
      window.location.href = `/rooms/${roomId}`;
    } catch (error) {
      console.error('Failed to join room:', error);
    }
  }
  
  function openCreateModal() {
    showCreateModal = true;
    roomName = '';
    selectedGameId = '';
    maxPlayers = 4;
  }
  
  function closeCreateModal() {
    showCreateModal = false;
  }
  
  function getGameName(gameId: string): string {
    return games.find(g => g.id === gameId)?.name || 'Unknown Game';
  }
</script>

<div class="room-lobby">
  <div class="header">
    <h1>Game Rooms</h1>
    <button class="btn-primary" onclick={openCreateModal}>
      Create Room
    </button>
  </div>
  
  {#if loading}
    <div class="loading">Loading rooms...</div>
  {:else if rooms.length === 0}
    <div class="empty-state">
      <p>No rooms available. Create one to get started!</p>
    </div>
  {:else}
    <div class="rooms-grid">
      {#each rooms as room}
        <div class="room-card">
          <div class="room-header">
            <h3>{room.name}</h3>
            <span class="game-name">{getGameName(room.game_id)}</span>
          </div>
          <div class="room-info">
            <span class="players">
              {room.current_players}/{room.max_players} players
            </span>
            <span class="status status-{room.status}">{room.status}</span>
          </div>
          <div class="room-actions">
            {#if room.status === 'waiting' && room.current_players < room.max_players}
              <button class="btn-secondary" onclick={() => joinRoom(room.id)}>
                Join Room
              </button>
            {:else}
              <button class="btn-disabled" disabled>
                {room.status === 'playing' ? 'In Progress' : 'Full'}
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCreateModal}
  <div class="modal-overlay" onclick={closeCreateModal}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>Create New Room</h2>
        <button class="close-btn" onclick={closeCreateModal}>×</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label for="game-select">Game</label>
          <select id="game-select" bind:value={selectedGameId}>
            <option value="">Select a game</option>
            {#each games as game}
              <option value={game.id}>{game.name}</option>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label for="room-name">Room Name</label>
          <input
            id="room-name"
            type="text"
            bind:value={roomName}
            placeholder="Enter room name"
          />
        </div>
        <div class="form-group">
          <label for="max-players">Max Players</label>
          <input
            id="max-players"
            type="number"
            bind:value={maxPlayers}
            min="2"
            max="8"
          />
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn-secondary" onclick={closeCreateModal}>Cancel</button>
        <button class="btn-primary" onclick={createRoom}>Create Room</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .room-lobby {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }
  
  .rooms-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
  }
  
  .room-card {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 1.5rem;
    background: white;
  }
  
  .room-header {
    margin-bottom: 1rem;
  }
  
  .room-header h3 {
    margin: 0 0 0.5rem 0;
    color: #333;
  }
  
  .game-name {
    color: #666;
    font-size: 0.9rem;
  }
  
  .room-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
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
  
  .btn-primary, .btn-secondary, .btn-disabled {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
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
  
  .btn-disabled {
    background: #f5f5f5;
    color: #999;
    cursor: not-allowed;
  }
  
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .modal {
    background: white;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
  }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #e0e0e0;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
  }
  
  .modal-body {
    padding: 1.5rem;
  }
  
  .form-group {
    margin-bottom: 1rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
  }
  
  .form-group input,
  .form-group select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
  }
  
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 1px solid #e0e0e0;
  }
  
  .loading, .empty-state {
    text-align: center;
    padding: 3rem;
    color: #666;
  }
</style>
