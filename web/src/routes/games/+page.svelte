<script lang="ts">
    import { gameService } from "$lib/services/games";
    import { tournamentService } from "$lib/services/tournaments";
    import { onMount } from "svelte";
    import type { Game, Tournament } from "$lib/types";
    import { LinkButton, Button } from "$lib/components";
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
            <div class="border p-6 shadow-sm bg-hatch bg-red-100 mb-4">
                <p class="text-red-600">{error}</p>
            </div>
        {/if}
        {#if loading}
            <div class="border border-[--border-color] p-6 shadow-sm bg-hatch text-center">
                <p class="text-gray-500">Loading games...</p>
            </div>
        {:else if games.length === 0}
            <div class="border border-[--border-color] p-6 shadow-sm bg-hatch text-center">
                <p class="text-gray-500">No games available</p>
            </div>
        {:else}
            <div class="grid grid-2">
                {#each games as game}
                    {#if game.is_active}
                        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
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
                                    class="grid gap-2 grid-cols-[auto_1fr]"
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
                            <div class="flex gap-2">
                                {#if tournamentsByGame.get(game.id)?.length}
                                    <LinkButton
                                        href="/tournaments?game={game.id}"
                                        variant="primary"
                                        label="View Tournaments"
                                    />
                                {:else}
                                    <Button
                                        variant="secondary"
                                        label="No Tournaments Yet"
                                        disabled={true}
                                    />
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
                                <div class="border border-[--border-color] p-6 shadow-sm bg-hatch opacity-70">
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
