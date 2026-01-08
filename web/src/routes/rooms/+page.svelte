<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { roomService } from "$lib/services/rooms";
    import { gameService } from "$lib/services/games";
    import type { Room, Game, CreateRoomRequest } from "$lib/types";
    import Dialog from "$lib/components/Dialog.svelte";
    import Alert from "$lib/components/Alert.svelte";
    import Button from "$lib/components/Button.svelte";
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

    async function joinRoom(roomId: string) {
        try {
            error = null;

            // Find the room to check if user is already in it
            const room = rooms.find(r => r.id === roomId);
            const currentUser = authState.user;

            if (room && currentUser) {
                // Check if user is already in the room (as host or player)
                const isAlreadyInRoom = room.host_id === currentUser.id || room.players.includes(currentUser.id);

                if (isAlreadyInRoom) {
                    // User is already in room, navigate directly
                    goto(`/room?id=${roomId}`);
                    return;
                }
            }

            // User is not in room, call join API
            await roomService.join(roomId);
            goto(`/room?id=${roomId}`);
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to join room";
            console.error("Failed to join room:", err);
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

<section class="container">
    <div class="flex justify-between items-center mb-4">
        <h1 class="text-2xl font-bold">Rooms</h1>
        <Button onclick={openCreateModal} label="+ Create Room" variant="primary" />
    </div>
    <div class="max-w-7xl mx-auto p-8">
        {#if error}
            <Alert
                type="error"
                message={error}
                onclose={() => (error = null)}
            />
        {/if}

        <div class="flex gap-4 items-center mb-8 p-4 bg-blueprint-line-faint">
            <label
                for="game-filter"
                class="block mb-2 font-medium text-blueprint-ink"
                >Filter by game:</label
            >
            <select
                id="game-filter"
                bind:value={filterGameId}
                onchange={loadData}
                class="p-2 min-w-50"
            >
                <option value="">All Games</option>
                {#each games as game}
                    <option value={game.id}>{game.name}</option>
                {/each}
            </select>
            <Button variant="secondary" label="Refresh" onclick={loadData} />
        </div>

        {#if loading}
            <div class="text-center p-12 text-blueprint-ink-light">Loading rooms...</div>
        {:else if rooms.length === 0}
            <div class="text-center p-12 text-blueprint-ink-light">
                <p>No rooms available. Create one to get started!</p>
            </div>
        {:else}
            <div class="grid grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-6">
                {#each rooms as room}
                    <div class="rounded-lg p-6 bg-blueprint-paper">
                        <div class="mb-4">
                            <h3 class="mb-2 text-blueprint-ink">{room.name}</h3>
                            <span class="text-blueprint-ink-light text-sm"
                                >{getGameName(room.game_id)}</span
                            >
                        </div>
                        <div class="flex justify-between items-center mb-4">
                            <span>
                                👥 {room.players.length}/{room.max_players}
                            </span>
                            <span class="px-2 py-1 text-xs capitalize {room.status === 'waiting' ? 'bg-blue-100 text-blue-800' : room.status === 'playing' ? 'bg-orange-100 text-orange-800' : 'bg-green-100 text-green-800'}"
                                >{room.status}</span
                            >
                        </div>
                        <div>
                            {#if room.status === "waiting"}
                                {#if isUserInRoom(room)}
                                    <Button
                                        variant="primary"
                                        label="Enter Room"
                                        onclick={() => joinRoom(room.id)}
                                    />
                                {:else if room.players.length < room.max_players}
                                    <Button
                                        variant="success"
                                        label="Join Room"
                                        onclick={() => joinRoom(room.id)}
                                    />
                                {:else}
                                    <Button
                                        variant="secondary"
                                        label="Full"
                                        disabled={true}
                                    />
                                {/if}
                            {:else}
                                <Button
                                    variant="secondary"
                                    label={room.status === "playing"
                                        ? "In Progress"
                                        : "Finished"}
                                    disabled={true}
                                />
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
        <div class="mb-4">
            <label for="game-select" class="block mb-2 font-medium text-blueprint-ink"
                >Game</label
            >
            <select id="game-select" required bind:value={selectedGameId} class="w-full p-2 bg-blueprint-paper">
                <option value="">Select a game</option>
                {#each games as game}
                    <option value={game.id}>{game.name}</option>
                {/each}
            </select>
        </div>
        <div class="mb-4">
            <label for="room-name" class="block mb-2 font-medium text-blueprint-ink"
                >Room Name</label
            >
            <input
                id="room-name"
                type="text"
                required
                bind:value={roomName}
                placeholder="Enter room name"
                class="w-full p-2 bg-blueprint-paper"
            />
        </div>
        <div class="mb-4">
            <label for="max-players" class="block mb-2 font-medium text-blueprint-ink"
                >Max Players</label
            >
            <input
                id="max-players"
                type="number"
                bind:value={maxPlayers}
                min="2"
                max="8"
                class="w-full p-2 bg-blueprint-paper"
            />
        </div>
        <div class="mb-4">
            <label for="human-timeout" class="block mb-2 font-medium text-blueprint-ink"
                >Human Timeout (ms)</label
            >
            <input
                id="human-timeout"
                type="number"
                bind:value={humanTimeoutMs}
                placeholder={getSelectedGame()?.human_timeout_ms?.toString() || "Default"}
                min="1000"
                class="w-full p-2 bg-blueprint-paper"
            />
            <p class="text-sm text-blueprint-ink-light mt-1">
                Leave empty to use game default ({getSelectedGame()?.human_timeout_ms || 'N/A'}ms)
            </p>
        </div>
    {/snippet}
</Dialog>
