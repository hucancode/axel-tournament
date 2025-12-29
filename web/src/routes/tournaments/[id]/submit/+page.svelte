<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { submissionService } from "$lib/services/submissions";
    import { tournamentService } from "$lib/services/tournaments";
    import { Button, LinkButton } from "$lib/components";
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
    const tournamentId = $derived(page.params.id!);
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
            goto(`/tournaments/${tournamentId}`);
        } catch (err) {
            error =
                err instanceof Error ? err.message : "Failed to submit code";
        } finally {
            loading = false;
        }
    }
</script>

<div class="container page">
    <div class="page-header">
        <h1 class="page-title">Submit Code</h1>
        {#if tournament}
            <p class="text-gray-500">Tournament: {tournament.name}</p>
        {/if}
    </div>
    {#if initialLoading}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch text-center">
            <p class="text-gray-500">Loading tournament...</p>
        </div>
    {:else}
        {#if error}
            <div
                class="border p-6 shadow-sm bg-hatch bg-red-100 border-l-4 border-red-600 mb-4"
            >
                <p class="text-red-600">{error}</p>
            </div>
        {/if}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
            {#if !isParticipant}
                <div
                    class="text-sm text-red-600 mb-4"
                >
                    You must join this tournament before submitting code.
                    <a
                        href="/tournaments/{tournamentId}"
                        class="ml-1 text-primary-600"
                    >
                        Go back
                    </a>
                </div>
            {/if}
            <form onsubmit={handleSubmit}>
                <div class="mb-4">
                    <label for="language" class="block mb-2 font-medium text-gray-dark">Programming Language</label>
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
                <div class="mb-4">
                    <label for="code" class="block mb-2 font-medium text-gray-dark">Code</label>
                    <textarea
                        id="code"
                        class="textarea font-mono text-sm"
                        bind:value={code}
                        disabled={loading || !isParticipant}
                        rows="25"
                        placeholder="Paste your code here..."
                    ></textarea>
                    {#if validationErrors.code}
                        <p class="form-error">{validationErrors.code}</p>
                    {/if}
                    <p
                        class="text-sm text-gray-500 mt-2"
                    >
                        {code.length.toLocaleString()} characters
                    </p>
                </div>
                <div class="flex gap-2">
                    <Button
                        variant="primary"
                        label={loading ? "Submitting..." : "Submit Code"}
                        disabled={loading || !isParticipant}
                    />
                    <LinkButton
                        href="/tournaments/{tournamentId}"
                        variant="secondary"
                        label="Cancel"
                    />
                </div>
            </form>
        </div>
    {/if}
</div>
