<script lang="ts">
    import { gameService } from "$services/games";
    import { tournamentService } from "$services/tournaments";
    import { onMount } from "svelte";
    import type { Game, Tournament } from "$lib/models";
    import { LinkButton, Card, Badge, Alert } from "$components";

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

<style>
    main {
        padding: var(--spacing-8) 0;
    }

    .page-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-4);
    }

    .page-header h1 {
        font-size: 1.5rem;
        font-weight: bold;
    }

    .loading-section, .empty-section {
        text-align: center;
    }

    .games-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: var(--spacing-4);
    }

    .game-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-2);
    }

    .game-header h2 {
        font-size: 1.25rem;
        font-weight: 600;
    }

    .game-description {
        color: var(--color-muted);
        margin-bottom: var(--spacing-4);
    }

    .game-languages, .game-stats {
        margin-bottom: var(--spacing-4);
    }

    .game-languages h3, .game-stats h3 {
        font-size: 0.875rem;
        font-weight: 600;
        color: var(--color-muted);
        margin-bottom: var(--spacing-2);
    }

    .language-badges {
        display: flex;
        gap: var(--spacing-2);
    }

    .stats-grid {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: var(--spacing-2);
    }

    .stats-grid dt {
        font-size: 0.875rem;
        color: var(--color-muted);
    }

    .stats-grid dd {
        font-size: 0.875rem;
        font-weight: 600;
    }

    .game-actions {
        display: flex;
        gap: var(--spacing-2);
        flex-wrap: wrap;
    }
</style>

<main>
    <div class="container">
        <header class="page-header">
            <h1>Available Games</h1>
        </header>

        {#if error}
            <Alert message={error} />
        {/if}

        {#if loading}
            <section class="loading-section">
                <Card class="loading-card">
                    <p>Loading games...</p>
                </Card>
            </section>
        {:else if games.length === 0}
            <section class="empty-section">
                <Card class="empty-card">
                    <p>No games available</p>
                </Card>
            </section>
        {:else}
            <section class="games-grid">
                {#each games as game}
                    <article class="game-card">
                        <Card>
                            <header class="game-header">
                                <h2>{game.name}</h2>
                            </header>
                            <p class="game-description">{game.description}</p>

                            <div class="game-languages">
                                <h3>Supported Languages:</h3>
                                <div class="language-badges">
                                    {#each game.supported_languages as lang}
                                        <Badge status="scheduled" label={lang.toUpperCase()} />
                                    {/each}
                                </div>
                            </div>

                            <div class="game-stats">
                                <h3>Tournament Statistics:</h3>
                                <dl class="stats-grid">
                                    <dt>Active Tournaments:</dt>
                                    <dd>{getActiveTournamentCount(game.id)}</dd>
                                    <dt>Total Tournaments:</dt>
                                    <dd>{getTotalTournamentCount(game.id)}</dd>
                                </dl>
                            </div>

                            <div class="game-actions">
                                {#if tournamentsByGame.get(game.id)?.length}
                                    <LinkButton
                                        href="/tournaments?game={game.id}"
                                        variant="primary"
                                        label="View Tournaments"
                                    />
                                {:else}
                                    <button data-variant="secondary" disabled>
                                        No Tournaments Yet
                                    </button>
                                {/if}
                            </div>
                        </Card>
                    </article>
                {/each}
            </section>
        {/if}
    </div>
</main>
