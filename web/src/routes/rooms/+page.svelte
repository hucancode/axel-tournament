<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { roomService } from "$services/rooms";
    import { gameService } from "$services/games";
    import type { Room, Game, CreateRoomRequest } from "$lib/types";
    import Dialog from "$lib/components/Dialog.svelte";
    import Alert from "$lib/components/Alert.svelte";
    import LinkButton from "$lib/components/LinkButton.svelte";
    import { authStore } from "$lib/stores/auth";

    let rooms = $state<Room[]>([]);
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let createDialog = $state<HTMLDialogElement | null>(null);
    let selectedGameId = $state("");
    let roomName = $state("");
    let maxPlayers = $state(4);
    let humanTimeoutMs = $state<number | undefined>(undefined);
    let filterGameId = $state<string>("");

    // Reactive auth state
    let authState = $state($authStore);

    onMount(async () => {
        // Subscribe to auth store changes
        authStore.subscribe(state => {
            authState = state;
        });
        await loadData();
    });

    async function loadData() {
        try {
            loading = true;
            error = null;
            const [roomsData, gamesData] = await Promise.all([
                roomService.list(filterGameId || undefined),
                gameService.list(),
            ]);
            rooms = roomsData;
            games = gamesData;
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load data";
            console.error("Failed to load data:", err);
        } finally {
            loading = false;
        }
    }

    async function createRoom() {
        try {
            error = null;
            const request: CreateRoomRequest = {
                game_id: selectedGameId,
                name: roomName.trim(),
                max_players: maxPlayers,
                human_timeout_ms: humanTimeoutMs,
            };
            const newRoom = await roomService.create(request);
            await loadData();
            closeCreateModal();
            goto(`/room?id=${newRoom.id}`);
        } catch (err) {
            error = err instanceof Error ? err.message : "";
            console.error("Failed to create room:", err);
        }
    }

    function openCreateModal() {
        if (!createDialog) return;
        createDialog.returnValue = "cancel";
        createDialog.showModal();
        roomName = "";
        selectedGameId = "";
        maxPlayers = 4;
        humanTimeoutMs = undefined;
    }

    function closeCreateModal() {
        createDialog?.close();
    }

    function onDialogClose() {
        if (createDialog?.returnValue === "submit") {
            createRoom();
        }
    }

    function getGameName(gameId: string): string {
        return games.find((g) => g.id === gameId)?.name || "Unknown Game";
    }

    function getSelectedGame(): Game | undefined {
        return games.find((g) => g.id === selectedGameId);
    }

    function isUserInRoom(room: Room): boolean {
        const currentUser = authState.user;
        if (!currentUser) return false;
        return room.host_id === currentUser.id || room.players.includes(currentUser.id);
    }
</script>

<style>
    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .header h1 {
        font-size: 1.5rem;
        font-weight: bold;
    }

    .content {
        max-width: 80rem;
        margin: 0 auto;
        padding: 2rem;
    }

    .filter-section {
        display: flex;
        gap: 1rem;
        align-items: center;
        margin-bottom: 2rem;
        padding: 1rem;
        background-color: var(--blueprint-line-faint);
    }

    .filter-section label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        color: var(--blueprint-ink);
    }

    .filter-select {
        padding: 0.5rem;
        min-width: 12.5rem;
    }

    .loading, .empty-state {
        text-align: center;
        padding: 3rem;
        color: var(--blueprint-ink-light);
    }

    .rooms-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .room-card {
        border-radius: 0.5rem;
        padding: 1.5rem;
        background-color: var(--blueprint-paper);
    }

    .room-info {
        margin-bottom: 1rem;
    }

    .room-info h3 {
        margin-bottom: 0.5rem;
        color: var(--blueprint-ink);
    }

    .game-name {
        color: var(--blueprint-ink-light);
        font-size: 0.875rem;
    }

    .room-meta {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .status-badge {
        padding: 0.25rem 0.5rem;
        font-size: 0.75rem;
        text-transform: capitalize;
    }

    .status-waiting {
        background-color: #dbeafe;
        color: #1e40af;
    }

    .status-playing {
        background-color: #fed7aa;
        color: #c2410c;
    }

    .status-finished {
        background-color: #dcfce7;
        color: #166534;
    }

    .form-field {
        margin-bottom: 1rem;
    }

    .form-field label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        color: var(--blueprint-ink);
    }

    .form-input {
        width: 100%;
        padding: 0.5rem;
        background-color: var(--blueprint-paper);
    }

    .form-help {
        font-size: 0.875rem;
        color: var(--blueprint-ink-light);
        margin-top: 0.25rem;
    }
</style>

<section class="container">
    <div class="header">
        <h1>Rooms</h1>
        <button onclick={openCreateModal} data-variant="primary">+ Create Room</button>
    </div>
    <div class="content">
        {#if error}
            <Alert
                type="error"
                message={error}
                onclose={() => (error = null)}
            />
        {/if}

        <div class="filter-section">
            <label for="game-filter">Filter by game:</label>
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
            <button data-variant="secondary" onclick={loadData}>Refresh</button>
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
                        <div class="room-info">
                            <h3>{room.name}</h3>
                            <span class="game-name">{getGameName(room.game_id)}</span>
                        </div>
                        <div class="room-meta">
                            <span>👥 {room.players.length}/{room.max_players}</span>
                            <span class="status-badge status-{room.status}">{room.status}</span>
                        </div>
                        <div>
                            {#if room.status === "waiting"}
                                {#if isUserInRoom(room)}
                                    <button
                                        data-variant="primary"
                                        onclick={() => goto(`/room?id=${room.id}`)}
                                    >
                                        Enter Room
                                    </button>
                                {:else if room.players.length < room.max_players}
                                    <button
                                        data-variant="success"
                                        onclick={() => goto(`/room?id=${room.id}`)}
                                    >
                                        Join Room
                                    </button>
                                {:else}
                                    <button
                                        data-variant="secondary"
                                        disabled
                                    >
                                        Full
                                    </button>
                                {/if}
                            {:else}
                                <button
                                    data-variant="secondary"
                                    disabled
                                >
                                    {room.status === "playing" ? "In Progress" : "Finished"}
                                </button>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</section>
<Dialog
    bind:dialog={createDialog}
    title="Create New Room"
    onclose={onDialogClose}
>
    {#snippet children()}
        <div class="form-field">
            <label for="game-select">Game</label>
            <select id="game-select" required bind:value={selectedGameId} class="form-input">
                <option value="">Select a game</option>
                {#each games as game}
                    <option value={game.id}>{game.name}</option>
                {/each}
            </select>
        </div>
        <div class="form-field">
            <label for="room-name">Room Name</label>
            <input
                id="room-name"
                type="text"
                required
                bind:value={roomName}
                placeholder="Enter room name"
                class="form-input"
            />
        </div>
        <div class="form-field">
            <label for="max-players">Max Players</label>
            <input
                id="max-players"
                type="number"
                bind:value={maxPlayers}
                min="2"
                max="8"
                class="form-input"
            />
        </div>
        <div class="form-field">
            <label for="human-timeout">Human Timeout (ms)</label>
            <input
                id="human-timeout"
                type="number"
                bind:value={humanTimeoutMs}
                placeholder={getSelectedGame()?.human_timeout_ms?.toString() || "Default"}
                min="1000"
                class="form-input"
            />
            <p class="form-help">
                Leave empty to use game default ({getSelectedGame()?.human_timeout_ms || 'N/A'}ms)
            </p>
        </div>
    {/snippet}
</Dialog>
