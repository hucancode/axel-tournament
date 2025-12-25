<script lang="ts">
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import { submissionService } from "$lib/services/submissions";
    import { authStore } from "$lib/stores/auth";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import type {
        Tournament,
        TournamentParticipant,
        Game,
        Submission,
    } from "$lib/types";
    const tournamentId = $derived(page.params.id!);
    let tournament = $state<Tournament | null>(null);
    let game = $state<Game | null>(null);
    let participants = $state<TournamentParticipant[]>([]);
    let userSubmissions = $state<Submission[]>([]);
    let loading = $state(true);
    let submissionsLoading = $state(false);
    let actionLoading = $state(false);
    let error = $state("");
    let actionError = $state("");
    let submissionError = $state("");
    let isParticipant = $state(false);
    onMount(async () => {
        await loadTournamentData();
    });
    async function loadTournamentData() {
        loading = true;
        error = "";
        try {
            // Load tournament and participants in parallel
            const [tournamentData, participantsData] = await Promise.all([
                tournamentService.get(tournamentId),
                tournamentService.getParticipants(tournamentId),
            ]);
            tournament = tournamentData;
            participants = participantsData;
            // Check if current user is a participant
            if ($authStore.isAuthenticated && $authStore.user) {
                isParticipant = participants.some(
                    (p) => p.user_id === $authStore.user!.id,
                );
                await loadUserSubmissions();
            } else {
                userSubmissions = [];
            }
            // Load game information
            if (tournament.game_id) {
                game = await gameService.get(tournament.game_id);
            }
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load tournament";
            console.error("Failed to load tournament:", err);
        } finally {
            loading = false;
        }
    }
    async function loadUserSubmissions() {
        submissionsLoading = true;
        submissionError = "";
        userSubmissions = [];
        try {
            userSubmissions = await submissionService.list(tournamentId);
        } catch (err) {
            submissionError =
                err instanceof Error
                    ? err.message
                    : "Failed to load your submissions";
        } finally {
            submissionsLoading = false;
        }
    }
    async function joinTournament() {
        if (!$authStore.isAuthenticated) {
            goto("/login");
            return;
        }
        actionLoading = true;
        actionError = "";
        try {
            await tournamentService.join(tournamentId);
            await loadTournamentData(); // Reload to update participant list
        } catch (err) {
            actionError =
                err instanceof Error
                    ? err.message
                    : "Failed to join tournament";
            console.error("Failed to join tournament:", err);
        } finally {
            actionLoading = false;
        }
    }
    async function leaveTournament() {
        if (!$authStore.isAuthenticated) {
            goto("/login");
            return;
        }
        actionLoading = true;
        actionError = "";
        try {
            await tournamentService.leave(tournamentId);
            await loadTournamentData(); // Reload to update participant list
        } catch (err) {
            actionError =
                err instanceof Error
                    ? err.message
                    : "Failed to leave tournament";
            console.error("Failed to leave tournament:", err);
        } finally {
            actionLoading = false;
        }
    }
    function getStatusBadgeClass(status: string): string {
        return `badge badge-${status}`;
    }
    function formatDate(dateStr?: string): string {
        if (!dateStr) return "Not set";
        return new Date(dateStr).toLocaleDateString("en-US", {
            month: "long",
            day: "numeric",
            year: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }
    function canJoin(): boolean {
        if (!tournament || !$authStore.isAuthenticated) return false;
        if (isParticipant) return false;
        if (participants.length >= tournament.max_players) return false;
        if (tournament.status !== "registration") return false;
        return true;
    }
    function canLeave(): boolean {
        if (!tournament || !$authStore.isAuthenticated) return false;
        if (!isParticipant) return false;
        if (
            tournament.status === "running" ||
            tournament.status === "completed"
        )
            return false;
        return true;
    }
    function canSubmit(): boolean {
        if (!tournament || !$authStore.isAuthenticated) return false;
        if (!isParticipant) return false;
        if (
            tournament.status === "completed" ||
            tournament.status === "cancelled"
        )
            return false;
        return true;
    }
</script>

<div class="page">
    <div class="container">
        {#if loading}
            <div class="card text-center">
                <p class="text-gray-500">Loading tournament...</p>
            </div>
        {:else if error}
            <div class="card" style="background: #fee2e2;">
                <p class="text-red-600">{error}</p>
            </div>
        {:else if tournament}
            <div class="page-header">
                <div class="flex justify-between items-center">
                    <div>
                        <h1 class="page-title">{tournament.name}</h1>
                        <span class={getStatusBadgeClass(tournament.status)}>
                            {tournament.status}
                        </span>
                    </div>
                    <a href="/tournaments" class="btn btn-secondary"
                        >Back to Tournaments</a
                    >
                </div>
            </div>
            {#if actionError}
                <div class="card mb-4" style="background: #fee2e2;">
                    <p class="text-red-600">{actionError}</p>
                </div>
            {/if}
            <div
                class="grid"
                style="grid-template-columns: 2fr 1fr; gap: 1.5rem;"
            >
                <div class="flex-col gap-4" style="display: flex;">
                    <!-- Tournament Details -->
                    <div class="card">
                        <h2 class="text-xl font-semibold mb-4">
                            About This Tournament
                        </h2>
                        <p class="text-gray-700 mb-4">
                            {tournament.description}
                        </p>
                        <div
                            class="grid gap-2"
                            style="grid-template-columns: auto 1fr;"
                        >
                            <div class="font-semibold text-gray-700">
                                Status:
                            </div>
                            <div>
                                <span
                                    class={getStatusBadgeClass(
                                        tournament.status,
                                    )}
                                >
                                    {tournament.status}
                                </span>
                            </div>
                            <div class="font-semibold text-gray-700">
                                Players:
                            </div>
                            <div>
                                {participants.length} / {tournament.max_players}
                                (min: {tournament.min_players})
                            </div>
                            {#if tournament.start_time}
                                <div class="font-semibold text-gray-700">
                                    Start Time:
                                </div>
                                <div>{formatDate(tournament.start_time)}</div>
                            {/if}
                            {#if tournament.end_time}
                                <div class="font-semibold text-gray-700">
                                    End Time:
                                </div>
                                <div>{formatDate(tournament.end_time)}</div>
                            {/if}
                            <div class="font-semibold text-gray-700">
                                Created:
                            </div>
                            <div>{formatDate(tournament.created_at)}</div>
                        </div>
                    </div>
                    <!-- Game Details -->
                    {#if game}
                        <div class="card">
                            <h2 class="text-xl font-semibold mb-4">
                                Game: {game.name}
                            </h2>
                            <p class="text-gray-700 mb-4">{game.description}</p>
                            <div class="mb-4">
                                <div class="font-semibold text-gray-700 mb-2">
                                    Supported Languages:
                                </div>
                                <div class="flex gap-2">
                                    {#each game.supported_languages as lang}
                                        <span class="badge badge-scheduled"
                                            >{lang}</span
                                        >
                                    {/each}
                                </div>
                            </div>
                        </div>
                    {/if}
                    <!-- Participants -->
                    <div class="card">
                        <h2 class="text-xl font-semibold mb-4">
                            Participants ({participants.length})
                        </h2>
                        {#if participants.length === 0}
                            <p class="text-center text-gray-500">
                                No participants yet
                            </p>
                        {:else}
                            <table>
                                <thead>
                                    <tr>
                                        <th>Rank</th>
                                        <th>Username</th>
                                        <th>Score</th>
                                        <th>Joined</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each participants as participant}
                                        <tr>
                                            <td>
                                                {#if participant.rank}
                                                    <span class="font-semibold"
                                                        >#{participant.rank}</span
                                                    >
                                                {:else}
                                                    <span class="text-gray-500"
                                                        >-</span
                                                    >
                                                {/if}
                                            </td>
                                            <td class="font-semibold">
                                                {participant.username ||
                                                    "Unknown"}
                                                {#if $authStore.user && participant.user_id === $authStore.user.id}
                                                    <span
                                                        class="badge badge-registration"
                                                        style="margin-left: 0.5rem;"
                                                        >You</span
                                                    >
                                                {/if}
                                            </td>
                                            <td>{participant.score || 0}</td>
                                            <td class="text-sm text-gray-500">
                                                {formatDate(
                                                    participant.joined_at,
                                                )}
                                            </td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        {/if}
                    </div>
                    <!-- User submissions -->
                    <div class="card">
                        <div class="flex justify-between items-center mb-3">
                            <h2 class="text-xl font-semibold">
                                Your Submissions
                            </h2>
                            {#if canSubmit()}
                                <a
                                    href="/tournaments/{tournamentId}/submit"
                                    class="btn btn-primary"
                                    style="padding: 0.35rem 0.75rem; font-size: 0.9rem;"
                                >
                                    New Submission
                                </a>
                            {/if}
                        </div>
                        {#if !$authStore.isAuthenticated}
                            <p class="text-sm text-gray-500">
                                Login to view and submit your code.
                            </p>
                        {:else if submissionsLoading}
                            <p class="text-sm text-gray-500">
                                Loading your submissions...
                            </p>
                        {:else if submissionError}
                            <p class="text-sm text-red-600">
                                {submissionError}
                            </p>
                        {:else if userSubmissions.length === 0}
                            <p class="text-sm text-gray-500">
                                You have not submitted any code yet.
                            </p>
                        {:else}
                            <table>
                                <thead>
                                    <tr>
                                        <th>#</th>
                                        <th>Language</th>
                                        <th>Status</th>
                                        <th>Submitted</th>
                                        <th>Notes</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each userSubmissions as submission, index}
                                        <tr>
                                            <td>#{index + 1}</td>
                                            <td class="font-semibold">
                                                {submission.language.toUpperCase()}
                                            </td>
                                            <td>
                                                <span
                                                    class="badge badge-{submission.status}"
                                                >
                                                    {submission.status}
                                                </span>
                                            </td>
                                            <td class="text-sm text-gray-500">
                                                {formatDate(
                                                    submission.created_at,
                                                )}
                                            </td>
                                            <td class="text-sm text-gray-600">
                                                {submission.error_message ||
                                                    "-"}
                                            </td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        {/if}
                    </div>
                </div>
                <!-- Actions Sidebar -->
                <div class="flex-col gap-4" style="display: flex;">
                    <div class="card">
                        <h3 class="font-semibold mb-4">Actions</h3>
                        {#if !$authStore.isAuthenticated}
                            <p class="text-sm text-gray-500 mb-4">
                                You must be logged in to participate
                            </p>
                            <a
                                href="/login"
                                class="btn btn-primary"
                                style="width: 100%;"
                            >
                                Login
                            </a>
                        {:else}
                            {#if canJoin()}
                                <button
                                    onclick={joinTournament}
                                    class="btn btn-success mb-2"
                                    style="width: 100%;"
                                    disabled={actionLoading}
                                >
                                    {actionLoading
                                        ? "Joining..."
                                        : "Join Tournament"}
                                </button>
                            {/if}
                            {#if canLeave()}
                                <button
                                    onclick={leaveTournament}
                                    class="btn btn-danger mb-2"
                                    style="width: 100%;"
                                    disabled={actionLoading}
                                >
                                    {actionLoading
                                        ? "Leaving..."
                                        : "Leave Tournament"}
                                </button>
                            {/if}
                            {#if canSubmit()}
                                <a
                                    href="/tournaments/{tournamentId}/submit"
                                    class="btn btn-primary"
                                    style="width: 100%;"
                                >
                                    Submit Code
                                </a>
                            {/if}
                            {#if isParticipant}
                                <div
                                    class="mt-4"
                                    style="padding: 1rem; background: var(--primary-50); border-radius: 0.5rem;"
                                >
                                    <p
                                        class="text-sm text-center font-semibold"
                                        style="color: var(--primary-700);"
                                    >
                                        You are participating in this tournament
                                    </p>
                                </div>
                            {/if}
                        {/if}
                    </div>
                    {#if tournament.status === "registration"}
                        <div
                            class="card"
                            style="background: var(--primary-50);"
                        >
                            <h3
                                class="font-semibold mb-2"
                                style="color: var(--primary-700);"
                            >
                                Registration Open
                            </h3>
                            <p
                                class="text-sm"
                                style="color: var(--primary-700);"
                            >
                                Join now to participate in this tournament!
                            </p>
                        </div>
                    {:else if tournament.status === "running"}
                        <div class="card" style="background: #fef3c7;">
                            <h3
                                class="font-semibold mb-2"
                                style="color: #92400e;"
                            >
                                Tournament In Progress
                            </h3>
                            <p class="text-sm" style="color: #92400e;">
                                This tournament is currently running. New
                                participants cannot join.
                            </p>
                        </div>
                    {:else if tournament.status === "completed"}
                        <div class="card" style="background: #d1fae5;">
                            <h3
                                class="font-semibold mb-2"
                                style="color: #065f46;"
                            >
                                Tournament Completed
                            </h3>
                            <p class="text-sm" style="color: #065f46;">
                                Check the participants list for final rankings.
                            </p>
                        </div>
                    {/if}
                </div>
            </div>
        {:else}
            <div class="card text-center">
                <p class="text-gray-500">Tournament not found</p>
            </div>
        {/if}
    </div>
</div>
