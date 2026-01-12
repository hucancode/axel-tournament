<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { submissionService } from "$services/submissions";
    import { tournamentService } from "$services/tournaments";
    import { LinkButton } from "$components";
    import type {
        Tournament,
        TournamentParticipant,
        ProgrammingLanguage,
    } from "$lib/types";
    let tournament = $state<Tournament | null>(null);
    let language = $state<ProgrammingLanguage>("rust");
    let code = $state("");
    let loading = $state(false);
    let initialLoading = $state(true);
    let error = $state("");
    let isParticipant = $state(false);
    let validationErrors = $state<{ language?: string; code?: string }>({});
    const tournamentId = $derived(page.url.searchParams.get('id') || '');
    const auth = $derived($authStore);
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
            goto(`/tournaments/tournament?id=${tournamentId}`);
        } catch (err) {
            error =
                err instanceof Error ? err.message : "Failed to submit code";
        } finally {
            loading = false;
        }
    }
</script>

<style>
    main {
        padding: var(--spacing-8) 0;
    }

    .page-header {
        margin-bottom: var(--spacing-4);
    }

    .tournament-name {
        color: var(--color-muted);
    }

    .loading-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        text-align: center;
        color: var(--color-muted);
    }

    .error-section {
        border: 1px solid var(--color-error);
        padding: var(--spacing-6);
        background-color: var(--color-gray-50);
        border-left: 4px solid var(--color-error);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .submit-form-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
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
        color: var(--color-gray-dark);
    }

    .code-textarea {
        font-family: monospace;
        font-size: 0.875rem;
    }

    .character-count {
        font-size: 0.875rem;
        color: var(--color-muted);
        margin-top: var(--spacing-2);
    }

    .form-actions {
        display: flex;
        gap: var(--spacing-2);
    }
</style>

<main>
    <div class="container">
        <header class="page-header">
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
                        <a href="/tournaments/tournament?id={tournamentId}">Go back</a>
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
                        <label for="code">Code</label>
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
                            href="/tournaments/tournament?id={tournamentId}"
                            variant="secondary"
                            label="Cancel"
                        />
                    </div>
                </form>
            </section>
        {/if}
    </div>
</main>
