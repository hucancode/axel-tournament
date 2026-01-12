<script lang="ts">
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
    import { authStore } from "$lib/stores/auth";
    import { onMount } from "svelte";
    import type { Tournament, TournamentParticipant, Game } from "$lib/types";
    import { LinkButton, Card, Badge } from "$lib/components";

    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
    let participantCounts = $state<Record<string, TournamentParticipant[]>>({});
    let loading = $state(true);
    let error = $state("");
    let selectedStatus = $state<string>("all");

    const statusOptions = [
        { value: "all", label: "All Tournaments" },
        { value: "scheduled", label: "Scheduled" },
        { value: "registration", label: "Registration Open" },
        { value: "generating", label: "Generating Matches" },
        { value: "running", label: "Running" },
        { value: "completed", label: "Completed" },
        { value: "cancelled", label: "Cancelled" },
    ];

    const auth = $derived($authStore);
    const canManageTournaments = $derived(auth.isAuthenticated && auth.user?.role === "admin");

    onMount(async () => {
        await loadTournaments();
    });

    async function loadTournaments() {
        loading = true;
        error = "";
        try {
            const status = selectedStatus === "all" ? undefined : selectedStatus;
            const [tournamentsData, gamesData] = await Promise.all([
                tournamentService.list(status),
                gameService.list(),
            ]);
            tournaments = tournamentsData;
            games = gamesData;

            // Load participants for each tournament
            const participantPromises = tournaments.map(async (tournament) => {
                try {
                    const participants = await tournamentService.getParticipants(tournament.id);
                    return { tournamentId: tournament.id, participants };
                } catch (err) {
                    console.error(`Failed to load participants for tournament ${tournament.id}:`, err);
                    return { tournamentId: tournament.id, participants: [] };
                }
            });

            const participantResults = await Promise.all(participantPromises);
            participantCounts = participantResults.reduce(
                (acc, { tournamentId, participants }) => {
                    acc[tournamentId] = participants;
                    return acc;
                },
                {} as Record<string, TournamentParticipant[]>,
            );
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load tournaments";
            console.error("Failed to load tournaments:", err);
        } finally {
            loading = false;
        }
    }

    async function handleStatusChange() {
        await loadTournaments();
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

    .filter-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        margin-bottom: var(--spacing-4);
    }

    .filter-controls {
        display: flex;
        align-items: center;
        gap: var(--spacing-4);
    }

    .filter-controls label {
        font-weight: 600;
    }

    .status-select {
        width: auto;
    }

    .error-section {
        padding: var(--spacing-6);
        background-color: var(--color-gray-50);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .loading-section, .empty-section {
        text-align: center;
    }

    .tournaments-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: var(--spacing-4);
    }

    .player-count {
        color: var(--color-muted);
    }
</style>

<main>
    <div class="container">
        <header class="page-header">
            <h1>Tournaments</h1>
            {#if canManageTournaments}
                <LinkButton href="/tournaments/new" label="+ Create Tournament" variant="primary" />
            {/if}
        </header>

        <section class="filter-section">
            <div class="filter-controls">
                <label for="status-filter">Filter by Status:</label>
                <select
                    id="status-filter"
                    class="status-select"
                    bind:value={selectedStatus}
                    onchange={handleStatusChange}
                    disabled={loading}
                >
                    {#each statusOptions as option}
                        <option value={option.value}>{option.label}</option>
                    {/each}
                </select>
            </div>
        </section>

        {#if error}
            <section class="error-section">
                <p>{error}</p>
            </section>
        {/if}

        {#if loading}
            <section class="loading-section">
                <Card class="loading-card">
                    <p>Loading tournaments...</p>
                </Card>
            </section>
        {:else if tournaments.length === 0}
            <section class="empty-section">
                <Card class="empty-card">
                    <p>No tournaments found</p>
                </Card>
            </section>
        {:else}
            <section class="tournaments-grid">
                {#each tournaments as tournament}
                    <Card href="/tournaments/tournament?id={tournament.id}">
                        <h3>{tournament.name}</h3>
                        <p>{tournament.description}</p>
                        <footer>
                            <Badge status={tournament.status} label={tournament.status} />
                            <span class="player-count">
                                {(participantCounts[tournament.id] || []).length}/{tournament.max_players} players
                            </span>
                        </footer>
                    </Card>
                {/each}
            </section>
        {/if}
    </div>
</main>
