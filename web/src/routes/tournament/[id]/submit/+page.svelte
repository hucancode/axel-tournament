<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { onMount, untrack } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { submissionService } from "$services/submissions";
    import { tournamentService } from "$services/tournaments";
    import { gameService, type StarterCode } from "$services/games";
    import { LinkButton } from "$components";
    import type {
        Tournament,
        TournamentParticipant,
        ProgrammingLanguage,
    } from "$lib/models";
    let tournament = $state<Tournament | null>(null);
    let language = $state<ProgrammingLanguage>("rust");
    let code = $state("");
    let loading = $state(false);
    let initialLoading = $state(true);
    let error = $state("");
    let isParticipant = $state(false);
    let validationErrors = $state<{ language?: string; code?: string }>({});
    let starter = $state<StarterCode | null>(null);
    let starterLoading = $state(false);
    let starterError = $state<string | null>(null);
    const tournamentId = $derived(page.params.id ?? "");
    const auth = $derived($authStore);

    async function loadStarter(gameId: string, lang: ProgrammingLanguage) {
        starterLoading = true;
        starterError = null;
        const prevStarterCode = starter?.code ?? null;
        try {
            const next = await gameService.getStarter(gameId, lang);
            starter = next;
            // Replace textarea only when the user hasn't touched it
            // (empty, or still matching the previously loaded starter).
            if (code === "" || code === prevStarterCode) {
                code = next.code;
            }
        } catch (err) {
            starter = null;
            starterError =
                err instanceof Error
                    ? err.message
                    : "Starter code unavailable";
            // If textarea was holding the prior starter, clear it so the
            // user isn't stuck with a stale template for a different language.
            if (prevStarterCode !== null && code === prevStarterCode) {
                code = "";
            }
        } finally {
            starterLoading = false;
        }
    }

    function useStarter() {
        if (starter) {
            code = starter.code;
        }
    }

    $effect(() => {
        if (tournament && language) {
            const gameId = tournament.game_id;
            const lang = language;
            untrack(() => loadStarter(gameId, lang));
        }
    });

    onMount(async () => {
        // Redirect if not authenticated
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        // Load tournament details
        try {
            const [tournamentData, participantData] = await Promise.all([
                tournamentService.get(tournamentId),
                tournamentService.getParticipants(tournamentId),
            ]);
            tournament = tournamentData;
            if (auth.user) {
                isParticipant = participantData.some(
                    (p: TournamentParticipant) =>
                        p.user_id === auth.user!.id,
                );
            }
            if (!isParticipant) {
                error =
                    "Join this tournament before submitting your solution.";
            }
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load tournament";
        } finally {
            initialLoading = false;
        }
    });
    function validate(): boolean {
        validationErrors = {};
        let isValid = true;
        if (!language) {
            validationErrors.language = "Please select a language";
            isValid = false;
        }
        if (!code.trim()) {
            validationErrors.code = "Please enter your code";
            isValid = false;
        }
        if (code.length > 1000000) {
            validationErrors.code = "Code must be less than 1MB";
            isValid = false;
        }
        return isValid;
    }
    async function handleSubmit(e: Event) {
        e.preventDefault();
        if (!isParticipant) {
            error = "Join this tournament before submitting your solution.";
            return;
        }
        if (!validate()) {
            return;
        }
        loading = true;
        error = "";
        try {
            await submissionService.create({
                tournament_id: tournamentId,
                language,
                code,
            });
            // Redirect to tournament page on success
            goto(`/tournament/${tournamentId}`);
        } catch (err) {
            error =
                err instanceof Error ? err.message : "Failed to submit code";
        } finally {
            loading = false;
        }
    }
</script>

<style>
    header {
        margin-bottom: var(--spacing-4);
    }

    .tournament-name {
        color: var(--color-fg-muted);
    }

    .loading-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
        text-align: center;
        color: var(--color-fg-muted);
    }

    .error-section {
        border: 1px solid var(--color-error);
        padding: var(--spacing-6);
        background-color: var(--color-bg-popup);
        border-left: 4px solid var(--color-error);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .submit-form-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
    }

    .participation-warning {
        font-size: 0.875rem;
        color: var(--color-error);
        margin-bottom: var(--spacing-4);
    }

    .participation-warning a {
        margin-left: var(--spacing-1);
        color: var(--color-primary);
        text-decoration: none;
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

    .code-textarea {
        font-family: monospace;
        font-size: 0.875rem;
    }

    .code-label-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--spacing-2);
        margin-bottom: var(--spacing-2);
        flex-wrap: wrap;
    }

    .code-label-row label {
        margin-bottom: 0;
    }

    .starter-controls {
        display: flex;
        align-items: center;
        gap: var(--spacing-2);
        font-size: 0.8125rem;
    }

    .starter-hint {
        color: var(--color-fg-muted);
    }

    .starter-hint code {
        font-family: monospace;
        background-color: var(--color-bg-popup);
        padding: 0 var(--spacing-1);
        border-radius: 2px;
    }

    .starter-hint-warn {
        color: var(--color-warning, var(--color-fg-muted));
    }

    .starter-reset {
        font-size: 0.8125rem;
    }

    .character-count {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
        margin-top: var(--spacing-2);
    }

    .form-actions {
        display: flex;
        gap: var(--spacing-2);
    }

    .playground-callout {
        margin-top: var(--spacing-4);
        padding: var(--spacing-3) var(--spacing-4);
        background-color: var(--color-bg-popup);
        border-left: 4px solid var(--color-primary);
        font-size: 0.875rem;
    }

    .playground-callout p {
        margin: 0 0 var(--spacing-2) 0;
        color: var(--color-fg-muted);
    }
</style>

<main>
    <div class="container">
        <header>
            <h1>Submit Code</h1>
            {#if tournament}
                <p class="tournament-name">Tournament: {tournament.name}</p>
            {/if}
        </header>

        {#if initialLoading}
            <section class="loading-section">
                <p>Loading tournament...</p>
            </section>
        {:else}
            {#if error}
                <section class="error-section">
                    <p>{error}</p>
                </section>
            {/if}

            <section class="submit-form-section">
                {#if !isParticipant}
                    <div class="participation-warning">
                        You must join this tournament before submitting code.
                        <a href="/tournament/{tournamentId}">Go back</a>
                    </div>
                {/if}

                <form onsubmit={handleSubmit} class="submit-form">
                    <div class="form-field">
                        <label for="language">Programming Language</label>
                        <select
                            id="language"
                            class="select"
                            bind:value={language}
                            disabled={loading || !isParticipant}
                        >
                            <option value="rust">Rust</option>
                            <option value="go">Go</option>
                            <option value="c">C</option>
                        </select>
                        {#if validationErrors.language}
                            <p class="form-error">{validationErrors.language}</p>
                        {/if}
                    </div>

                    <div class="form-field">
                        <div class="code-label-row">
                            <label for="code">Code</label>
                            <div class="starter-controls">
                                {#if starterLoading}
                                    <span class="starter-hint">Loading starter code…</span>
                                {:else if starter}
                                    <span class="starter-hint">
                                        Starter loaded from <code>{starter.filename}</code>
                                    </span>
                                    <button
                                        type="button"
                                        data-variant="ghost"
                                        class="starter-reset"
                                        onclick={useStarter}
                                        disabled={loading || !isParticipant}
                                    >
                                        Reset to starter
                                    </button>
                                {:else if starterError}
                                    <span class="starter-hint starter-hint-warn">
                                        No starter code available for this game + language
                                    </span>
                                {/if}
                            </div>
                        </div>
                        <textarea
                            id="code"
                            class="code-textarea"
                            bind:value={code}
                            disabled={loading || !isParticipant}
                            rows="25"
                            placeholder="Paste your code here..."
                        ></textarea>
                        {#if validationErrors.code}
                            <p class="form-error">{validationErrors.code}</p>
                        {/if}
                        <p class="character-count">
                            {code.length.toLocaleString()} characters
                        </p>
                    </div>

                    <div class="form-actions">
                        <button
                            type="submit"
                            data-variant="primary"
                            disabled={loading || !isParticipant}
                        >
                            {loading ? "Submitting..." : "Submit Code"}
                        </button>
                        <LinkButton
                            href="/tournament/{tournamentId}"
                            variant="secondary"
                            label="Cancel"
                        />
                    </div>
                </form>

                {#if tournament}
                    <aside class="playground-callout">
                        <p>
                            New to the protocol? Open the playground and act as
                            a bot against a built-in sample. Every wire frame
                            between the server and both clients is shown live.
                        </p>
                        <LinkButton
                            href="/playground?game={tournament.game_id}"
                            variant="secondary"
                            label="Open protocol playground"
                        />
                    </aside>
                {/if}
            </section>
        {/if}
    </div>
</main>
