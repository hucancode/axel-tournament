<script lang="ts">
    import { gameService } from "$lib/services/games";
    import { tournamentService } from "$lib/services/tournaments";
    import { onMount } from "svelte";
    import type { Game, Tournament } from "$lib/types";
    let games = $state<Game[]>([]);
    let tournamentsByGame = $state<Map<string, Tournament[]>>(new Map());
    let loading = $state(true);
    let error = $state("");
    onMount(async () => {
        await loadGames();
    });
    async function loadGames() {
        loading = true;
        error = "";
        try {
            // Load all games
            games = await gameService.list();
            // Load all tournaments to group by game
            const allTournaments = await tournamentService.list();
            // Group tournaments by game_id
            const groupedTournaments = new Map<string, Tournament[]>();
            for (const tournament of allTournaments) {
                if (!groupedTournaments.has(tournament.game_id)) {
                    groupedTournaments.set(tournament.game_id, []);
                }
                groupedTournaments.get(tournament.game_id)!.push(tournament);
            }
            tournamentsByGame = groupedTournaments;
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load games";
            console.error("Failed to load games:", err);
        } finally {
            loading = false;
        }
    }
    function getActiveTournamentCount(gameId: string): number {
        const tournaments = tournamentsByGame.get(gameId) || [];
        return tournaments.filter(
            (t) => t.status === "registration" || t.status === "running",
        ).length;
    }
    function getTotalTournamentCount(gameId: string): number {
        return tournamentsByGame.get(gameId)?.length || 0;
    }
</script>

<div class="page">
    <div class="container">
        <div class="page-header">
            <h1 class="page-title">Games</h1>
            <p class="text-gray-500">Available programming challenge games</p>
        </div>
        {#if error}
            <div class="card" style="background: #fee2e2; margin-bottom: 1rem;">
                <p class="text-red-600">{error}</p>
            </div>
        {/if}
        {#if loading}
            <div class="card text-center">
                <p class="text-gray-500">Loading games...</p>
            </div>
        {:else if games.length === 0}
            <div class="card text-center">
                <p class="text-gray-500">No games available</p>
            </div>
        {:else}
            <div class="grid grid-2">
                {#each games as game}
                    {#if game.is_active}
                        <div class="card">
                            <div class="flex justify-between items-center mb-2">
                                <h2 class="text-xl font-semibold">
                                    {game.name}
                                </h2>
                                <span class="badge badge-accepted">Active</span>
                            </div>
                            <p class="text-gray-700 mb-4">{game.description}</p>
                            <div class="mb-4">
                                <div
                                    class="text-sm font-semibold text-gray-700 mb-2"
                                >
                                    Supported Languages:
                                </div>
                                <div class="flex gap-2">
                                    {#each game.supported_languages as lang}
                                        <span class="badge badge-scheduled"
                                            >{lang.toUpperCase()}</span
                                        >
                                    {/each}
                                </div>
                            </div>
                            <div class="mb-4">
                                <div
                                    class="text-sm font-semibold text-gray-700 mb-2"
                                >
                                    Tournament Statistics:
                                </div>
                                <div
                                    class="grid gap-2"
                                    style="grid-template-columns: auto 1fr;"
                                >
                                    <div class="text-sm text-gray-500">
                                        Active Tournaments:
                                    </div>
                                    <div class="text-sm font-semibold">
                                        {getActiveTournamentCount(game.id)}
                                    </div>
                                    <div class="text-sm text-gray-500">
                                        Total Tournaments:
                                    </div>
                                    <div class="text-sm font-semibold">
                                        {getTotalTournamentCount(game.id)}
                                    </div>
                                </div>
                            </div>
                            {#if game.rules && Object.keys(game.rules).length > 0}
                                <div class="mb-4">
                                    <div
                                        class="text-sm font-semibold text-gray-700 mb-2"
                                    >
                                        Game Rules:
                                    </div>
                                    <pre
                                        class="text-sm"
                                        style="background: var(--gray-100); padding: 0.75rem; border-radius: 0.5rem; overflow-x: auto; max-height: 200px;">{JSON.stringify(
                                            game.rules,
                                            null,
                                            2,
                                        )}</pre>
                                </div>
                            {/if}
                            <div class="flex gap-2">
                                {#if tournamentsByGame.get(game.id)?.length}
                                    <a
                                        href="/tournaments?game={game.id}"
                                        class="btn btn-primary"
                                        style="flex: 1;"
                                    >
                                        View Tournaments
                                    </a>
                                {:else}
                                    <button
                                        class="btn btn-secondary"
                                        style="flex: 1;"
                                        disabled
                                    >
                                        No Tournaments Yet
                                    </button>
                                {/if}
                            </div>
                            <div class="text-xs text-gray-500 mt-4">
                                Created {new Date(
                                    game.created_at,
                                ).toLocaleDateString()}
                            </div>
                        </div>
                    {/if}
                {/each}
            </div>
            <!-- Show inactive games separately if any exist -->
            {#if games.some((g) => !g.is_active)}
                <div class="mt-4">
                    <h2 class="text-xl font-semibold mb-4">Inactive Games</h2>
                    <div class="grid grid-3">
                        {#each games as game}
                            {#if !game.is_active}
                                <div class="card" style="opacity: 0.7;">
                                    <div
                                        class="flex justify-between items-center mb-2"
                                    >
                                        <h3 class="text-lg font-semibold">
                                            {game.name}
                                        </h3>
                                        <span class="badge badge-cancelled"
                                            >Inactive</span
                                        >
                                    </div>
                                    <p class="text-sm text-gray-600">
                                        {game.description}
                                    </p>
                                </div>
                            {/if}
                        {/each}
                    </div>
                </div>
            {/if}
        {/if}
    </div>
</div>
