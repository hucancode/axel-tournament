<script lang="ts">
    import { leaderboardService } from "$services/leaderboard";
    import { tournamentService } from "$services/tournaments";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { Alert, PageHeader, LinkButton } from "$components";
    import type { LeaderboardEntry, Tournament } from "$lib/models";

    const tournamentId = $derived(page.params.id ?? "");
    let entries = $state<LeaderboardEntry[]>([]);
    let tournament = $state<Tournament | null>(null);
    let loading = $state(true);
    let error = $state("");
    let limit = $state(100);

    onMount(async () => {
        await loadAll();
    });

    async function loadAll() {
        loading = true;
        error = "";
        try {
            const [tournamentData, entriesData] = await Promise.all([
                tournamentService.get(tournamentId),
                leaderboardService.get(tournamentId, limit),
            ]);
            tournament = tournamentData;
            entries = entriesData;
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

    async function reloadEntries() {
        loading = true;
        error = "";
        try {
            entries = await leaderboardService.get(tournamentId, limit);
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load leaderboard";
        } finally {
            loading = false;
        }
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
    .filters-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
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
        background-color: var(--color-bg-light);
    }

    .loading-section, .empty-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
        text-align: center;
        color: var(--color-fg-muted);
    }

    .empty-hint {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
        margin-top: var(--spacing-2);
    }

    .leaderboard-section {
        background-color: var(--color-bg-light);
        padding: 0;
        overflow-x: auto;
    }

    .table-container {
        overflow-x: auto;
    }

    .leaderboard-table {
        background-color: var(--color-bg-light);
    }

    .leaderboard-table thead {
        background-color: var(--color-border-light);
        position: sticky;
        top: 0;
        z-index: 10;
    }

    .leaderboard-table th {
        padding: var(--spacing-3);
        border-bottom: 2px solid var(--color-border-light);
    }

    .rank-column {
        width: 5rem;
    }

    .leaderboard-row:hover {
        background-color: var(--color-border-light);
    }

    .leaderboard-table td {
        padding: var(--spacing-3);
        border-bottom: 1px solid var(--color-border-light);
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
        color: var(--color-warning);
    }

    .rank-silver {
        color: var(--color-fg-dim);
    }

    .rank-bronze {
        color: var(--color-orange);
    }

    .player-cell {
        font-weight: 600;
        color: var(--color-border);
    }

    .no-location {
        color: var(--color-fg-muted);
    }

    .score-cell {
        font-weight: bold;
        font-size: 1.125rem;
        color: var(--color-primary);
    }

    .results-count {
        text-align: center;
        margin-top: var(--spacing-4);
        font-size: 0.875rem;
        color: var(--color-fg-muted);
    }

    .header-actions {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-4);
    }
</style>

<main>
    <div class="container">
        <PageHeader title={tournament ? `Leaderboard — ${tournament.name}` : "Leaderboard"} />
        <div class="header-actions">
            <LinkButton
                href="/tournament/{tournamentId}"
                variant="secondary"
                label="Back to Tournament"
            />
        </div>
        {#if error}
            <Alert message={error} />
        {/if}
        <section class="filters-section">
            <div class="filters-grid">
                <div class="filter-group">
                    <label for="limit-filter">Limit</label>
                    <select
                        id="limit-filter"
                        class="filter-select"
                        bind:value={limit}
                        onchange={reloadEntries}
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


        {#if loading}
            <section class="loading-section">
                <p>Loading leaderboard...</p>
            </section>
        {:else if entries.length === 0}
            <section class="empty-section">
                <p>No leaderboard entries found</p>
                <p class="empty-hint">Tournament has no scored participants yet</p>
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
                            </tr>
                        </thead>
                        <tbody>
                            {#each entries as entry}
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
