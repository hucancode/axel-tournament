<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { roomService } from '$lib/services/rooms';
  import { gameService } from '$lib/services/games';
  import type { Room, Game, CreateRoomRequest } from '$lib/types';
  import Dialog from '$lib/components/Dialog.svelte';
  import Alert from '$lib/components/Alert.svelte';

  let rooms = $state<Room[]>([]);
  let games = $state<Game[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let createDialog = $state<HTMLDialogElement | null>(null);
  let selectedGameId = $state('');
  let roomName = $state('');
  let maxPlayers = $state(4);
  let filterGameId = $state<string>('');

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = null;
      const [roomsData, gamesData] = await Promise.all([
        roomService.list(filterGameId || undefined),
        gameService.list()
      ]);
      rooms = roomsData;
      games = gamesData.filter(g => g.game_type === 'interactive');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load data';
      console.error('Failed to load data:', err);
    } finally {
      loading = false;
    }
  }

  async function createRoom() {
    if (!selectedGameId || !roomName.trim()) return;

    try {
      error = null;
      const request: CreateRoomRequest = {
        game_id: selectedGameId,
        name: roomName.trim(),
        max_players: maxPlayers
      };

      const newRoom = await roomService.create(request);
      await loadData();
      closeCreateModal();
      goto(`/rooms/${newRoom.id}`);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to create room';
      console.error('Failed to create room:', err);
    }
  }

  async function joinRoom(roomId: string) {
    try {
      error = null;
      await roomService.join(roomId);
      goto(`/rooms/${roomId}`);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to join room';
      console.error('Failed to join room:', err);
    }
  }

  function openCreateModal() {
    if (!createDialog) return;
    createDialog.returnValue = 'cancel';
    createDialog.showModal();
    roomName = '';
    selectedGameId = '';
    maxPlayers = 4;
  }

  function closeCreateModal() {
    createDialog?.close();
  }

  function onDialogClose() {
    if (createDialog?.returnValue === 'submit') {
      createRoom();
    }
  }

  function getGameName(gameId: string): string {
    return games.find(g => g.id === gameId)?.name || 'Unknown Game';
  }
</script>

<div class="room-lobby">
  <div class="header">
    <h1>Game Rooms</h1>
    <button class="btn-primary" onclick={openCreateModal}>
      + Create Room
    </button>
  </div>

  {#if error}
    <Alert type="error" message={error} onclose={() => error = null} />
  {/if}

  <div class="filter-bar">
    <label for="game-filter" class="block mb-2 font-medium text-gray-dark">Filter by game:</label>
    <select
      id="game-filter"
      bind:value={filterGameId}
      onchange={loadData}
      class="filter-select"
    >
      <option value="">All Games</option>
      {#each games as game}
        <option value={game.id}>{game.name}</option>
      {/each}
    </select>
    <button class="btn-secondary" onclick={loadData}>Refresh</button>
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
              👥 {room.players.length}/{room.max_players}
            </span>
            <span class="status status-{room.status}">{room.status}</span>
          </div>
          <div class="room-actions">
            {#if room.status === 'waiting' && room.players.length < room.max_players}
              <button class="btn-join" onclick={() => joinRoom(room.id)}>
                Join Room
              </button>
            {:else if room.status === 'waiting'}
              <button class="btn-disabled" disabled>Full</button>
            {:else}
              <button class="btn-disabled" disabled>
                {room.status === 'playing' ? 'In Progress' : 'Finished'}
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<Dialog bind:dialog={createDialog} title="Create New Room" onclose={onDialogClose}>
  <div class="mb-4">
    <label for="game-select" class="block mb-2 font-medium text-gray-dark">Game</label>
    <select id="game-select" bind:value={selectedGameId}>
      <option value="">Select a game</option>
      {#each games as game}
        <option value={game.id}>{game.name}</option>
      {/each}
    </select>
  </div>
  <div class="mb-4">
    <label for="room-name" class="block mb-2 font-medium text-gray-dark">Room Name</label>
    <input
      id="room-name"
      type="text"
      bind:value={roomName}
      placeholder="Enter room name"
    />
  </div>
  <div class="mb-4">
    <label for="max-players" class="block mb-2 font-medium text-gray-dark">Max Players</label>
    <input
      id="max-players"
      type="number"
      bind:value={maxPlayers}
      min="2"
      max="8"
    />
  </div>
</Dialog>

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
    font-size: 0.8rem;
    text-transform: capitalize;
  }

  .status-waiting { background: #e3f2fd; color: #1976d2; }
  .status-playing { background: #fff3e0; color: #f57c00; }
  .status-finished { background: #e8f5e8; color: #388e3c; }

  .filter-bar {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-bottom: 2rem;
    padding: 1rem;
    background: #f9f9f9;
  }

  .filter-bar label {
    font-weight: 500;
  }

  .filter-select {
    padding: 0.5rem;
    border: 1px solid #ddd;
    min-width: 200px;
  }

  .btn-primary, .btn-secondary, .btn-join, .btn-disabled {
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #1976d2;
    color: white;
  }

  .btn-primary:hover {
    background: #1565c0;
  }

  .btn-secondary {
    background: #f5f5f5;
    color: #333;
    border: 1px solid #ddd;
  }

  .btn-secondary:hover {
    background: #e0e0e0;
  }

  .btn-join {
    background: #4caf50;
    color: white;
  }

  .btn-join:hover {
    background: #45a049;
  }

  .btn-disabled {
    background: #f5f5f5;
    color: #999;
    cursor: not-allowed;
  }

  .loading, .empty-state {
    text-align: center;
    padding: 3rem;
    color: #666;
  }
</style>
