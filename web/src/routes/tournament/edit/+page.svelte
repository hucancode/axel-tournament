<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
    import { LinkButton, Card } from "$components";
    import type { Tournament, Game, UpdateTournamentRequest, TournamentStatus } from "$lib/models";
    import { configFieldsFor, defaultConfig } from "$lib/games/gameConfig";

    let tournamentId = $derived(page.url.searchParams.get('id') || '');
    let tournament = $state<Tournament | null>(null);
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");

    let formData = $state<UpdateTournamentRequest>({
        name: "",
        description: "",
        status: "scheduled",
        start_time: "",
        end_time: "",
    });
    let formLoading = $state(false);
    let formError = $state("");
    let configForm = $state<Record<string, unknown>>({});
    let configSaving = $state(false);
    const editableConfig = $derived(
        tournament?.status === "scheduled" || tournament?.status === "registration"
    );

    const auth = $derived($authStore);
    const statusOptions: { value: TournamentStatus; label: string }[] = [
        { value: "scheduled", label: "Scheduled" },
        { value: "registration", label: "Registration Open" },
        { value: "generating", label: "Generating Matches" },
        { value: "running", label: "Running" },
        { value: "completed", label: "Completed" },
        { value: "cancelled", label: "Cancelled" },
    ];

    onMount(async () => {
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        await loadData();
    });

    async function loadData() {
        loading = true;
        error = "";
        try {
            const [tournamentData, gamesData] = await Promise.all([
                tournamentService.get(tournamentId),
                gameService.list(),
            ]);
            tournament = tournamentData;
            games = gamesData;

            // Check permissions - only admins can edit tournaments
            if (auth.user?.role !== "admin") {
                goto("/tournaments");
                return;
            }

            formData = {
                name: tournament.name,
                description: tournament.description,
                status: tournament.status,
                start_time: tournament.start_time ? formatDateForInput(tournament.start_time) : "",
                end_time: tournament.end_time ? formatDateForInput(tournament.end_time) : "",
            };
            configForm = {
                ...defaultConfig(tournament.game_id),
                ...(tournament.config ?? {}),
            };
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load tournament";
        } finally {
            loading = false;
        }
    }

    function formatDateForInput(dateStr: string): string {
        return dateStr.slice(0, 16);
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";

        try {
            await tournamentService.update(tournamentId, formData);
            goto(`/tournament?id=${tournamentId}`);
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to update tournament";
        } finally {
            formLoading = false;
        }
    }

    async function saveConfig() {
        if (!tournament) return;
        configSaving = true;
        formError = "";
        try {
            const updated = await tournamentService.updateConfig(tournamentId, configForm);
            tournament = updated;
            configForm = { ...defaultConfig(updated.game_id), ...(updated.config ?? {}) };
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to update config";
        } finally {
            configSaving = false;
        }
    }

    async function handleStartTournament() {
        formLoading = true;
        formError = "";
        try {
            await tournamentService.start(tournamentId);
            goto(`/tournament?id=${tournamentId}`);
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to start tournament";
        } finally {
            formLoading = false;
        }
    }
</script>

<style>
    header {
        margin-bottom: var(--spacing-8);
    }

    .header-content {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .title-section h1 {
        font-size: 2rem;
    }

    .subtitle {
        color: var(--color-fg-muted);
    }

    .loading-section {
        text-align: center;
    }

    .error-section, .form-error-section {
        background-color: var(--color-bg-popup);
        border-left: 4px solid var(--color-error);
        padding: var(--spacing-4);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .tournament-form {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
    }

    .form-field {
        margin-bottom: var(--spacing-4);
    }

    .form-field label {
        display: block;
        margin-bottom: var(--spacing-2);
        font-weight: 500;
        color: var(--color-fg);
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

<main>
    <div class="container">
        <header>
            <div class="header-content">
                <div class="title-section">
                    <h1>Edit Tournament</h1>
                    <p class="subtitle">Update tournament settings</p>
                </div>
                <LinkButton variant="secondary" href="/tournament?id={tournamentId}" label="Back to Tournament" />
            </div>
        </header>

        {#if loading}
            <section class="loading-section">
                <Card class="loading-card">
                    <p>Loading tournament...</p>
                </Card>
            </section>
        {:else if error}
            <section class="error-section">
                <p>{error}</p>
            </section>
        {:else if tournament}
            {#if formError}
                <section class="form-error-section">
                    <p>{formError}</p>
                </section>
            {/if}

            <section class="form-section">
                <form onsubmit={handleSubmit} class="tournament-form">
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

                    <div class="form-field">
                        <label for="status">Status</label>
                        <select
                            id="status"
                            class="input"
                            bind:value={formData.status}
                            disabled={formLoading}
                            required
                        >
                            {#each statusOptions as option}
                                <option value={option.value}>{option.label}</option>
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

                    {#if tournament.game_id && configFieldsFor(tournament.game_id).length > 0}
                        <fieldset class="form-field" disabled={!editableConfig}>
                            <legend>Per-game settings</legend>
                            {#if !editableConfig}
                                <p class="subtitle">
                                    Locked — config can only be edited while the tournament is
                                    scheduled or in registration.
                                </p>
                            {/if}
                            {#each configFieldsFor(tournament.game_id) as f}
                                <div class="form-field">
                                    <label for="cfg-{f.key}">{f.label}</label>
                                    {#if f.type === 'boolean'}
                                        <input
                                            id="cfg-{f.key}"
                                            type="checkbox"
                                            checked={!!configForm[f.key]}
                                            onchange={(e) => {
                                                const v = (e.currentTarget as HTMLInputElement).checked;
                                                configForm = { ...configForm, [f.key]: v };
                                            }}
                                        />
                                    {:else}
                                        <input
                                            id="cfg-{f.key}"
                                            class="input"
                                            type="number"
                                            min={f.min}
                                            max={f.max}
                                            step={f.step ?? 1}
                                            value={(configForm[f.key] ?? f.default) as number}
                                            onchange={(e) => {
                                                const n = Number((e.currentTarget as HTMLInputElement).value);
                                                configForm = {
                                                    ...configForm,
                                                    [f.key]: Number.isFinite(n) ? n : f.default,
                                                };
                                            }}
                                        />
                                    {/if}
                                    {#if f.help}<p class="subtitle">{f.help}</p>{/if}
                                </div>
                            {/each}
                            {#if editableConfig}
                                <button
                                    type="button"
                                    data-variant="secondary"
                                    onclick={saveConfig}
                                    disabled={configSaving}
                                >
                                    {configSaving ? "Saving config…" : "Save per-game config"}
                                </button>
                            {/if}
                        </fieldset>
                    {/if}

                    <div class="form-actions">
                        <button
                            type="submit"
                            data-variant="primary"
                            disabled={formLoading}
                        >
                            {formLoading ? "Updating..." : "Update Tournament"}
                        </button>
                        {#if tournament.status === "registration"}
                            <button
                                type="button"
                                data-variant="success"
                                disabled={formLoading}
                                onclick={handleStartTournament}
                            >
                                Start Tournament
                            </button>
                        {/if}
                        <LinkButton
                            href="/tournament?id={tournamentId}"
                            variant="secondary"
                            label="Cancel"
                        />
                    </div>
                </form>
            </section>
        {/if}
    </div>
</main>
