<script lang="ts">
    import { gameService } from "$lib/services/games";
    import { tournamentService } from "$lib/services/tournaments";
    import { onMount } from "svelte";
    import type { Game, Tournament } from "$lib/types";
    import { LinkButton, Button, LoadingCard, EmptyState, Badge, Card } from "$lib/components";

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
            games = await gameService.list();
            const allTournaments = await tournamentService.list();
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

<section class="container">
    <div class="flex justify-between items-center mb-4">
        <h1 class="text-2xl font-bold">Available Games</h1>
    </div>

    {#if error}
        <div class="border p-6 bg-hatch bg-red-100 mb-4">
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <LoadingCard message="Loading games..." />
    {:else if games.length === 0}
        <EmptyState message="No games available" />
    {:else}
        <div class="grid grid-cols-2">
            {#each games as game}
                <Card>
                    <div class="flex justify-between items-center mb-2">
                        <h2 class="text-xl font-semibold">
                            {game.name}
                        </h2>
                    </div>
                    <p class="text-gray-700 mb-4">{game.description}</p>
                    <div class="mb-4">
                        <div class="text-sm font-semibold text-gray-700 mb-2">
                            Supported Languages:
                        </div>
                        <div class="flex gap-2">
                            {#each game.supported_languages as lang}
                                <Badge status="scheduled" label={lang.toUpperCase()} />
                            {/each}
                        </div>
                    </div>
                    <div class="mb-4">
                        <div class="text-sm font-semibold text-gray-700 mb-2">
                            Tournament Statistics:
                        </div>
                        <div class="grid gap-2 grid-cols-[auto_1fr]">
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
                    <div class="flex gap-2 flex-wrap">
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
                </Card>
            {/each}
        </div>
    {/if}
</section>
