<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
    import { LinkButton, Card } from "$lib/components";
    import type { Game, CreateTournamentRequest, MatchGenerationType } from "$lib/types";

    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");

    let formData = $state<CreateTournamentRequest>({
        game_id: "",
        name: "",
        description: "",
        min_players: 2,
        max_players: 100,
        start_time: "",
        end_time: "",
        match_generation_type: "all_vs_all",
    });
    let formLoading = $state(false);
    let formError = $state("");

    const auth = $derived($authStore);
    const matchTypes: { value: MatchGenerationType; label: string }[] = [
        { value: "all_vs_all", label: "All vs All" },
        { value: "round_robin", label: "Round Robin" },
        { value: "single_elimination", label: "Single Elimination" },
        { value: "double_elimination", label: "Double Elimination" },
    ];

    // Only admins can create tournaments
    const ownedGames = $derived(() => {
        if (!auth.isAuthenticated) return [];
        if (auth.user?.role === "admin") return games;
        return [];
    });

    onMount(async () => {
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        if (auth.user?.role !== "admin") {
            goto("/tournaments");
            return;
        }
        await loadGames();
    });

    async function loadGames() {
        loading = true;
        error = "";
        try {
            games = await gameService.list();
            const owned = ownedGames();
            if (owned.length > 0) {
                formData.game_id = owned[0].id;
            }
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load games";
        } finally {
            loading = false;
        }
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";

        if (formData.min_players < 2) {
            formError = "Minimum players must be at least 2";
            formLoading = false;
            return;
        }

        if (formData.max_players < formData.min_players) {
            formError = "Maximum players must be greater than or equal to minimum players";
            formLoading = false;
            return;
        }

        try {
            await tournamentService.create(formData);
            goto("/tournaments");
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to create tournament";
        } finally {
            formLoading = false;
        }
    }
</script>

<style>
    .tournament-create-page {
        padding: var(--spacing-8) 0;
    }

    .page-header {
        margin-bottom: var(--spacing-8);
    }

    .header-content {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .title-section h1 {
        font-size: 2rem;
        font-weight: bold;
        margin-bottom: var(--spacing-2);
    }

    .subtitle {
        color: var(--color-muted);
    }

    .loading-section, .no-games-section {
        text-align: center;
    }

    .error-section, .form-error-section {
        background-color: var(--color-gray-50);
        border-left: 4px solid var(--color-error);
        padding: var(--spacing-4);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .form-section {
        background-color: var(--color-blueprint-paper);
    }

    .tournament-form {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
    }

    .form-field {
        margin-bottom: var(--spacing-4);
    }

    .form-field label {
        display: block;
        margin-bottom: var(--spacing-2);
        font-weight: 500;
        color: var(--color-gray-dark);
    }

    .form-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-4);
        margin-bottom: var(--spacing-4);
    }

    .form-actions {
        display: flex;
        gap: var(--spacing-2);
    }
</style>

<main class="tournament-create-page">
    <div class="container">
        <header class="page-header">
            <div class="header-content">
                <div class="title-section">
                    <h1>Create New Tournament</h1>
                    <p class="subtitle">Set up a new competitive tournament</p>
                </div>
                <LinkButton variant="secondary" href="/tournaments" label="Back to Tournaments" />
            </div>
        </header>

        {#if loading}
            <section class="loading-section">
                <Card class="loading-card">
                    <p>Loading games...</p>
                </Card>
            </section>
        {:else if error}
            <section class="error-section">
                <p>{error}</p>
            </section>
        {:else if ownedGames().length === 0}
            <section class="no-games-section">
                <Card class="no-games-card">
                    <p>You need to create a game before creating tournaments.</p>
                    <button onclick={() => goto("/games/new")} data-variant="primary">Create Game</button>
                </Card>
            </section>
        {:else}
            {#if formError}
                <section class="form-error-section">
                    <p>{formError}</p>
                </section>
            {/if}

            <section class="form-section">
                <form onsubmit={handleSubmit} class="tournament-form">
                    <div class="form-field">
                        <label for="game">Game</label>
                        <select
                            id="game"
                            class="input"
                            bind:value={formData.game_id}
                            disabled={formLoading}
                            required
                        >
                            {#each ownedGames() as game}
                                <option value={game.id}>{game.name}</option>
                            {/each}
                        </select>
                    </div>

                    <div class="form-field">
                        <label for="name">Tournament Name</label>
                        <input
                            id="name"
                            type="text"
                            class="input"
                            bind:value={formData.name}
                            disabled={formLoading}
                            required
                        />
                    </div>

                    <div class="form-field">
                        <label for="description">Description</label>
                        <textarea
                            id="description"
                            class="textarea"
                            bind:value={formData.description}
                            disabled={formLoading}
                            rows="4"
                            required
                        ></textarea>
                    </div>

                    <div class="form-row">
                        <div class="form-field">
                            <label for="min-players">Minimum Players</label>
                            <input
                                id="min-players"
                                type="number"
                                class="input"
                                min="2"
                                bind:value={formData.min_players}
                                disabled={formLoading}
                                required
                            />
                        </div>

                        <div class="form-field">
                            <label for="max-players">Maximum Players</label>
                            <input
                                id="max-players"
                                type="number"
                                class="input"
                                min="2"
                                bind:value={formData.max_players}
                                disabled={formLoading}
                                required
                            />
                        </div>
                    </div>

                    <div class="form-field">
                        <label for="match-type">Match Generation Type</label>
                        <select
                            id="match-type"
                            class="input"
                            bind:value={formData.match_generation_type}
                            disabled={formLoading}
                            required
                        >
                            {#each matchTypes as type}
                                <option value={type.value}>{type.label}</option>
                            {/each}
                        </select>
                    </div>

                    <div class="form-row">
                        <div class="form-field">
                            <label for="start-time">Start Time (Optional)</label>
                            <input
                                id="start-time"
                                type="datetime-local"
                                class="input"
                                bind:value={formData.start_time}
                                disabled={formLoading}
                            />
                        </div>

                        <div class="form-field">
                            <label for="end-time">End Time (Optional)</label>
                            <input
                                id="end-time"
                                type="datetime-local"
                                class="input"
                                bind:value={formData.end_time}
                                disabled={formLoading}
                            />
                        </div>
                    </div>

                    <div class="form-actions">
                        <button
                            type="submit"
                            data-variant="primary"
                            disabled={formLoading}
                        >
                            {formLoading ? "Creating..." : "Create Tournament"}
                        </button>
                        <LinkButton
                            href="/tournaments"
                            variant="secondary"
                            label="Cancel"
                        />
                    </div>
                </form>
            </section>
        {/if}
    </div>
</main>
