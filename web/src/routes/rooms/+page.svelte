<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { roomService } from "$services/rooms";
    import { gameService } from "$services/games";
    import type { Room, Game, CreateRoomRequest } from "$lib/models";
    import { Alert, PageHeader } from "$components";
    import { authStore } from "$lib/stores/auth";
    import { configFieldsFor, defaultConfig } from "$lib/games/gameConfig";

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
    let createConfig = $state<Record<string, unknown>>({});

    /// Refresh the per-game config form whenever the user picks a different
    /// game in the create dialog. New game = new defaults.
    $effect(() => {
        if (selectedGameId) {
            createConfig = defaultConfig(selectedGameId);
        }
    });

    // Reactive auth state
    let authState = $state($authStore);

    onMount(async () => {
        // Subscribe to auth store changes
        authStore.subscribe((state) => {
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
                config: createConfig,
            };
            const newRoom = await roomService.create(request);
            await loadData();
            closeCreateModal();
            goto(`/room?id=${encodeURIComponent(newRoom.id)}&game=${encodeURIComponent(newRoom.game_id)}`);
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
        createConfig = {};
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

    async function joinAndEnter(room: Room) {
        try {
            error = null;
            await roomService.join(room.id);
            goto(`/room?id=${encodeURIComponent(room.id)}&game=${encodeURIComponent(room.game_id)}`);
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to join room";
        }
    }

    function isUserInRoom(room: Room): boolean {
        const currentUser = authState.user;
        if (!currentUser) return false;
        return (
            room.host_id === currentUser.id ||
            room.players.includes(currentUser.id)
        );
    }
</script>

<main>
    <section class="container">
        <PageHeader title="Rooms">
            <button onclick={openCreateModal} data-variant="primary"
                >+ Create Room</button
            >
        </PageHeader>
        <div class="content">
            {#if error}
                <Alert
                    type="error"
                    message={error}
                    onclose={() => (error = null)}
                />
            {/if}

            <div class="filter-section">
                <select
                    id="game-filter"
                    bind:value={filterGameId}
                    onchange={loadData}
                >
                    <option value="">All Games</option>
                    {#each games as game}
                        <option value={game.id}>{game.name}</option>
                    {/each}
                </select>
                <button data-variant="secondary" onclick={loadData}
                    >Refresh</button
                >
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
                                <h3>
                                    {room.name}
                                    {#if room.is_ranked}
                                        <span class="badge-ranked">RANKED</span>
                                    {/if}
                                </h3>
                                <span class="game-name"
                                    >{getGameName(room.game_id)}</span
                                >
                            </div>
                            <div class="room-meta">
                                <span
                                    >👥 {room.players
                                        .length}/{room.max_players}</span
                                >
                                <span class="status-badge status-{room.status}"
                                    >{room.status}</span
                                >
                            </div>
                            <div>
                                {#if room.status === "lobby"}
                                    {#if isUserInRoom(room)}
                                        <button
                                            data-variant="primary"
                                            onclick={() =>
                                                goto(`/room?id=${encodeURIComponent(room.id)}&game=${encodeURIComponent(room.game_id)}`)}
                                        >
                                            Enter Room
                                        </button>
                                    {:else if room.players.length < room.max_players}
                                        <button
                                            data-variant="success"
                                            onclick={() => joinAndEnter(room)}
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
                                    <button data-variant="secondary" disabled>
                                        {room.status === "playing"
                                            ? "In Progress"
                                            : "Finished"}
                                    </button>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    </section>
</main>
<dialog bind:this={createDialog} onclose={onDialogClose}>
    <form method="dialog">
        <header>
            <h2>Create New Room</h2>
            <button
                type="button"
                onclick={() => createDialog?.close()}
                aria-label="Close">×</button
            >
        </header>
        <div class="dialog-content">
            <div class="form-field">
                <label for="game-select">Game</label>
                <select
                    id="game-select"
                    required
                    bind:value={selectedGameId}
                    class="form-input"
                >
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
            {#if selectedGameId}
                {#each configFieldsFor(selectedGameId) as f}
                    <div class="form-field">
                        <label for="cfg-{f.key}">{f.label}</label>
                        {#if f.type === 'boolean'}
                            <input
                                id="cfg-{f.key}"
                                type="checkbox"
                                checked={!!createConfig[f.key]}
                                onchange={(e) => {
                                    const v = (e.currentTarget as HTMLInputElement).checked;
                                    createConfig = { ...createConfig, [f.key]: v };
                                }}
                            />
                        {:else}
                            <input
                                id="cfg-{f.key}"
                                type="number"
                                min={f.min}
                                max={f.max}
                                step={f.step ?? 1}
                                value={(createConfig[f.key] ?? f.default) as number}
                                class="form-input"
                                onchange={(e) => {
                                    const n = Number((e.currentTarget as HTMLInputElement).value);
                                    createConfig = {
                                        ...createConfig,
                                        [f.key]: Number.isFinite(n) ? n : f.default,
                                    };
                                }}
                            />
                        {/if}
                        {#if f.help}<p class="form-help">{f.help}</p>{/if}
                    </div>
                {/each}
            {/if}
            <div class="form-field">
                <label for="human-timeout">Human Timeout (ms)</label>
                <input
                    id="human-timeout"
                    type="number"
                    bind:value={humanTimeoutMs}
                    placeholder={getSelectedGame()?.human_timeout_ms?.toString() ||
                        "Default"}
                    min="1000"
                    class="form-input"
                />
                <p class="form-help">
                    Leave empty to use game default ({getSelectedGame()
                        ?.human_timeout_ms || "N/A"}ms)
                </p>
            </div>
        </div>
        <footer>
            <button
                type="button"
                data-variant="secondary"
                onclick={() => createDialog?.close()}
            >
                Cancel
            </button>
            <button type="submit" data-variant="primary" value="submit">
                Create
            </button>
        </footer>
    </form>
</dialog>

<style>
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
        background-color: var(--color-border-light);
    }

    .filter-select {
        padding: 0.5rem;
        min-width: 12.5rem;
    }

    .loading,
    .empty-state {
        text-align: center;
        padding: 3rem;
        color: var(--color-fg-muted);
    }

    .rooms-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .room-card {
        border-radius: 0.5rem;
        padding: 1.5rem;
        background-color: var(--color-bg-light);
    }

    .room-info {
        margin-bottom: 1rem;
    }

    .room-info h3 {
        margin-bottom: 0.5rem;
        color: var(--color-fg);
    }

    .game-name {
        color: var(--color-fg-muted);
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

    .status-lobby {
        background-color: var(--color-info);
        color: var(--color-bg);
    }

    .status-abandoned {
        background-color: var(--color-error);
        color: var(--color-bg);
    }

    .badge-ranked {
        background-color: var(--color-warning);
        color: var(--color-bg);
        padding: 0.15rem 0.4rem;
        font-size: 0.7rem;
        margin-left: 0.5rem;
    }

    .status-playing {
        background-color: var(--color-warning);
        color: var(--color-bg);
    }

    .status-finished {
        background-color: var(--color-success);
        color: var(--color-bg);
    }

    .form-field {
        margin-bottom: 1rem;
    }

    .form-field label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        color: var(--color-fg);
    }

    .form-input {
        width: 100%;
        padding: 0.5rem;
        background-color: var(--color-bg-light);
    }

    .form-help {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
        margin-top: 0.25rem;
    }
</style>
