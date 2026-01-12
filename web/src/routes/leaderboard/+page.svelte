<script lang="ts">
    import { leaderboardService } from "$services/leaderboard";
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
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
        if (rank === 1) return "rank-gold";
        if (rank === 2) return "rank-silver";
        if (rank === 3) return "rank-bronze";
        return "";
    }
</script>

<style>
    main {
        padding: var(--spacing-8) 0;
    }

    .filters-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        margin-bottom: var(--spacing-4);
    }

    .filters-section h2 {
        font-size: 1.125rem;
        font-weight: 600;
        margin-bottom: var(--spacing-4);
    }

    .filters-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: var(--spacing-4);
        margin-bottom: var(--spacing-6);
    }

    .filter-group label {
        display: block;
        margin-bottom: var(--spacing-2);
        font-weight: 500;
        color: var(--color-fg);
    }

    .filter-select {
        width: 100%;
        padding: var(--spacing-2);
        background-color: var(--color-blueprint-paper);
    }

    .error-section {
        border: 1px solid var(--color-error);
        padding: var(--spacing-6);
        background-color: var(--color-gray-50);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .loading-section, .empty-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        text-align: center;
        color: var(--color-muted);
    }

    .empty-hint {
        font-size: 0.875rem;
        color: var(--color-muted);
        margin-top: var(--spacing-2);
    }

    .leaderboard-section {
        background-color: var(--color-blueprint-paper);
        padding: 0;
        overflow-x: auto;
    }

    .table-container {
        overflow-x: auto;
    }

    .leaderboard-table {
        width: 100%;
        border-collapse: collapse;
        background-color: var(--color-blueprint-paper);
    }

    .leaderboard-table thead {
        background-color: var(--color-blueprint-line-faint);
        position: sticky;
        top: 0;
        z-index: 10;
    }

    .leaderboard-table th {
        padding: var(--spacing-3);
        text-align: left;
        font-weight: 600;
        border-bottom: 2px solid var(--color-blueprint-line-light);
    }

    .rank-column {
        width: 5rem;
    }

    .leaderboard-row:hover {
        background-color: var(--color-blueprint-line-faint);
    }

    .leaderboard-table td {
        padding: var(--spacing-3);
        border-bottom: 1px solid var(--color-blueprint-line-light);
    }

    .rank-cell {
        font-weight: bold;
        font-size: 1.125rem;
    }

    .rank-content {
        display: inline-flex;
        align-items: center;
        gap: var(--spacing-2);
    }

    .medal {
        font-size: 1.25rem;
    }

    .rank-gold {
        color: #d97706;
    }

    .rank-silver {
        color: #64748b;
    }

    .rank-bronze {
        color: #92400e;
    }

    .player-cell {
        font-weight: 600;
        color: var(--color-blueprint-line);
    }

    .no-location {
        color: var(--color-muted);
    }

    .score-cell {
        font-weight: bold;
        font-size: 1.125rem;
        color: var(--color-primary);
    }

    .tournament-link {
        font-size: 0.875rem;
        color: var(--color-primary);
        text-decoration: none;
    }

    .results-count {
        text-align: center;
        margin-top: var(--spacing-4);
        font-size: 0.875rem;
        color: var(--color-muted);
    }
</style>

<main>
    <div class="container">
        <section class="filters-section">
            <h2>Filters</h2>
            <div class="filters-grid">
                <div class="filter-group">
                    <label for="tournament-filter">Tournament</label>
                    <select
                        id="tournament-filter"
                        class="filter-select"
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
                <div class="filter-group">
                    <label for="game-filter">Game</label>
                    <select
                        id="game-filter"
                        class="filter-select"
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
                <div class="filter-group">
                    <label for="limit-filter">Limit</label>
                    <select
                        id="limit-filter"
                        class="filter-select"
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
        </section>

        {#if error}
            <section class="error-section">
                <p>{error}</p>
            </section>
        {/if}

        {#if loading}
            <section class="loading-section">
                <p>Loading leaderboard...</p>
            </section>
        {:else if entries.length === 0}
            <section class="empty-section">
                <p>No leaderboard entries found</p>
                <p class="empty-hint">Try adjusting your filters or check back later</p>
            </section>
        {:else}
            <section class="leaderboard-section">
                <div class="table-container">
                    <table class="leaderboard-table">
                        <thead>
                            <tr>
                                <th class="rank-column">Rank</th>
                                <th>Player</th>
                                <th>Location</th>
                                <th>Score</th>
                                <th>Tournament</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each entries as entry, index}
                                <tr class="leaderboard-row">
                                    <td class="rank-cell {getRankClass(entry.rank)}">
                                        <span class="rank-content">
                                            <span>{entry.rank}</span>
                                            <span class="medal">{getMedalEmoji(entry.rank)}</span>
                                        </span>
                                    </td>
                                    <td class="player-cell">{entry.username}</td>
                                    <td class="location-cell">
                                        {#if entry.location}
                                            <span class="badge badge-scheduled">{entry.location}</span>
                                        {:else}
                                            <span class="no-location">-</span>
                                        {/if}
                                    </td>
                                    <td class="score-cell">{entry.score.toLocaleString()}</td>
                                    <td class="tournament-cell">
                                        <a href="/tournaments/{entry.tournament_id}" class="tournament-link">
                                            {entry.tournament_name}
                                        </a>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
                <div class="results-count">
                    Showing {entries.length} {entries.length === 1 ? "entry" : "entries"}
                </div>
            </section>
        {/if}
    </div>
</main>
