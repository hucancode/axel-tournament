<script lang="ts">
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
    import { authStore } from "$lib/stores/auth";
    import { onMount } from "svelte";
    import type { Tournament, Game } from "$lib/models";
    import { LinkButton, Card, Badge, Alert, PageHeader } from "$components";

    const ACTIVE_STATUSES = [
        "scheduled",
        "registration",
        "generating",
        "running",
    ];
    const RECENT_COMPLETED_WINDOW_MS = 7 * 24 * 60 * 60 * 1000;

    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");
    let selectedStatus = $state<string>("default");

    const statusOptions = [
        { value: "default", label: "Open + Recently Finished" },
        { value: "all", label: "All Tournaments" },
        { value: "scheduled", label: "Scheduled" },
        { value: "registration", label: "Registration Open" },
        { value: "generating", label: "Generating Matches" },
        { value: "running", label: "Running" },
        { value: "completed", label: "Completed" },
        { value: "cancelled", label: "Cancelled" },
    ];

    const auth = $derived($authStore);
    const canManageTournaments = $derived(
        auth.isAuthenticated && auth.user?.role === "admin",
    );

    const visibleTournaments = $derived.by(() => {
        if (selectedStatus !== "default") return tournaments;
        const cutoff = Date.now() - RECENT_COMPLETED_WINDOW_MS;
        return tournaments.filter((t) => {
            if (ACTIVE_STATUSES.includes(t.status)) return true;
            if (t.status !== "completed") return false;
            const ts = Date.parse(t.end_time ?? t.updated_at);
            return Number.isFinite(ts) && ts >= cutoff;
        });
    });

    onMount(async () => {
        await loadTournaments();
    });

    async function loadTournaments() {
        loading = true;
        error = "";
        try {
            const status =
                selectedStatus === "all" || selectedStatus === "default"
                    ? undefined
                    : selectedStatus;
            const [tournamentsData, gamesData] = await Promise.all([
                tournamentService.list(status),
                gameService.list(),
            ]);
            tournaments = tournamentsData;
            games = gamesData;
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load tournaments";
            console.error("Failed to load tournaments:", err);
        } finally {
            loading = false;
        }
    }

    async function handleStatusChange() {
        await loadTournaments();
    }
</script>

<main>
    <div class="container">
        <PageHeader title="Tournaments">
            {#if canManageTournaments}
                <LinkButton
                    href="/tournaments/new"
                    label="+ Create Tournament"
                    variant="primary"
                />
            {/if}
        </PageHeader>

        {#if error}
            <Alert message={error} />
        {/if}

        <section class="filter-section">
            <div class="filter-controls">
                <select
                    id="status-filter"
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

        {#if loading}
            <section class="loading-section">
                <Card class="loading-card">
                    <p>Loading tournaments...</p>
                </Card>
            </section>
        {:else if visibleTournaments.length === 0}
            <section class="empty-section">
                <Card class="empty-card">
                    <p>No tournaments found</p>
                </Card>
            </section>
        {:else}
            <section class="tournaments-grid">
                {#each visibleTournaments as tournament}
                    <Card href="/tournament/{tournament.id}">
                        <h3>{tournament.name}</h3>
                        <p>{tournament.description}</p>
                        <footer>
                            <Badge
                                status={tournament.status}
                                label={tournament.status}
                            />
                            <span class="player-count">
                                {tournament.participant_count}/{tournament.max_players}
                                players
                            </span>
                        </footer>
                    </Card>
                {/each}
            </section>
        {/if}
    </div>
</main>

<style>
    .filter-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
        margin-bottom: var(--spacing-4);
    }

    .filter-controls {
        display: flex;
        align-items: center;
        gap: var(--spacing-4);
    }

    .loading-section,
    .empty-section {
        text-align: center;
    }

    .tournaments-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: var(--spacing-4);
    }

    .player-count {
        color: var(--color-fg-muted);
    }
</style>
