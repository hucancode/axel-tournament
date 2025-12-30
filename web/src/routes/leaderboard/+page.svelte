<script lang="ts">
    import { leaderboardService } from "$lib/services/leaderboard";
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import { onMount } from "svelte";
    import type { LeaderboardEntry, Tournament, Game } from "$lib/types";
    let entries = $state<LeaderboardEntry[]>([]);
    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");
    let selectedTournament = $state<string>("all");
    let selectedGame = $state<string>("all");
    let limit = $state(100);
    onMount(async () => {
        await loadFilters();
        await loadLeaderboard();
    });
    async function loadFilters() {
        try {
            // Load tournaments and games for filter dropdowns
            const [tournamentsData, gamesData] = await Promise.all([
                tournamentService.list(),
                gameService.list(),
            ]);
            tournaments = tournamentsData;
            games = gamesData;
        } catch (err) {
            console.error("Failed to load filters:", err);
        }
    }
    async function loadLeaderboard() {
        loading = true;
        error = "";
        try {
            const filters: {
                tournament_id?: string;
                game_id?: string;
                limit?: number;
            } = {
                limit,
            };
            if (selectedTournament !== "all") {
                filters.tournament_id = selectedTournament;
            }
            if (selectedGame !== "all") {
                filters.game_id = selectedGame;
            }
            entries = await leaderboardService.get(filters);
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load leaderboard";
            console.error("Failed to load leaderboard:", err);
        } finally {
            loading = false;
        }
    }
    async function handleFilterChange() {
        await loadLeaderboard();
    }
    function getMedalEmoji(rank: number): string {
        if (rank === 1) return "🥇";
        if (rank === 2) return "🥈";
        if (rank === 3) return "🥉";
        return "";
    }
    function getRankClass(rank: number): string {
        if (rank === 1) return "text-amber-600 font-bold";
        if (rank === 2) return "text-slate-500 font-bold";
        if (rank === 3) return "text-amber-800 font-bold";
        return "";
    }
</script>

<section class="container">
    <div class="border border-blueprint-line-light p-6 shadow-sm bg-hatch mb-4">
        <h2 class="text-lg font-semibold mb-4">Filters</h2>
        <div class="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-4 mb-6">
            <div>
                <label
                    for="tournament-filter"
                    class="block mb-2 font-medium text-blueprint-ink"
                    >Tournament</label
                >
                <select
                    id="tournament-filter"
                    class="w-full p-2 border border-blueprint-line-light bg-blueprint-paper"
                    bind:value={selectedTournament}
                    onchange={handleFilterChange}
                    disabled={loading}
                >
                    <option value="all">All Tournaments</option>
                    {#each tournaments as tournament}
                        <option value={tournament.id}>{tournament.name}</option>
                    {/each}
                </select>
            </div>
            <div>
                <label
                    for="game-filter"
                    class="block mb-2 font-medium text-blueprint-ink">Game</label
                >
                <select
                    id="game-filter"
                    class="w-full p-2 border border-blueprint-line-light bg-blueprint-paper"
                    bind:value={selectedGame}
                    onchange={handleFilterChange}
                    disabled={loading}
                >
                    <option value="all">All Games</option>
                    {#each games as game}
                        <option value={game.id}>{game.name}</option>
                    {/each}
                </select>
            </div>
            <div>
                <label
                    for="limit-filter"
                    class="block mb-2 font-medium text-blueprint-ink">Limit</label
                >
                <select
                    id="limit-filter"
                    class="w-full p-2 border border-blueprint-line-light bg-blueprint-paper"
                    bind:value={limit}
                    onchange={handleFilterChange}
                    disabled={loading}
                >
                    <option value={10}>Top 10</option>
                    <option value={25}>Top 25</option>
                    <option value={50}>Top 50</option>
                    <option value={100}>Top 100</option>
                </select>
            </div>
        </div>
    </div>
    {#if error}
        <div class="border border-red-200 p-6 shadow-sm bg-red-50 mb-4">
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <div
            class="border border-blueprint-line-light p-6 shadow-sm bg-hatch text-center"
        >
            <p class="text-blueprint-ink-light">Loading leaderboard...</p>
        </div>
    {:else if entries.length === 0}
        <div
            class="border border-blueprint-line-light p-6 shadow-sm bg-hatch text-center"
        >
            <p class="text-blueprint-ink-light">No leaderboard entries found</p>
            <p class="text-sm text-blueprint-ink-light mt-2">
                Try adjusting your filters or check back later
            </p>
        </div>
    {:else}
        <div
            class="border border-blueprint-line-light shadow-sm bg-hatch p-0 overflow-x-auto"
        >
            <table class="w-full border-collapse bg-blueprint-paper">
                <thead class="bg-blueprint-line-faint sticky top-0 z-10">
                    <tr>
                        <th class="p-3 text-left font-semibold border-b-2 border-blueprint-line-light w-20">Rank</th>
                        <th class="p-3 text-left font-semibold border-b-2 border-blueprint-line-light">Player</th>
                        <th class="p-3 text-left font-semibold border-b-2 border-blueprint-line-light">Location</th>
                        <th class="p-3 text-left font-semibold border-b-2 border-blueprint-line-light">Score</th>
                        <th class="p-3 text-left font-semibold border-b-2 border-blueprint-line-light">Tournament</th>
                    </tr>
                </thead>
                <tbody>
                    {#each entries as entry, index}
                        <tr class="hover:bg-blueprint-line-faint">
                            <td class="p-3 border-b border-blueprint-line-light font-bold text-lg {getRankClass(entry.rank)}">
                                <span class="inline-flex items-center gap-2">
                                    <span>{entry.rank}</span>
                                    <span class="text-xl"
                                        >{getMedalEmoji(entry.rank)}</span
                                    >
                                </span>
                            </td>
                            <td class="p-3 border-b border-blueprint-line-light font-semibold text-blueprint-line">{entry.username}</td>
                            <td class="p-3 border-b border-blueprint-line-light">
                                {#if entry.location}
                                    <span class="badge badge-scheduled"
                                        >{entry.location}</span
                                    >
                                {:else}
                                    <span class="text-blueprint-ink-light">-</span>
                                {/if}
                            </td>
                            <td class="p-3 border-b border-blueprint-line-light font-bold text-lg text-primary"
                                >{entry.score.toLocaleString()}</td
                            >
                            <td class="p-3 border-b border-blueprint-line-light">
                                <a
                                    href="/tournaments/{entry.tournament_id}"
                                    class="text-sm text-primary no-underline"
                                >
                                    {entry.tournament_name}
                                </a>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
        <div class="text-center mt-4 text-sm text-blueprint-ink-light">
            Showing {entries.length}
            {entries.length === 1 ? "entry" : "entries"}
        </div>
    {/if}
</section>
