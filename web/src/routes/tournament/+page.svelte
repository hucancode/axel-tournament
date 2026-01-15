<script lang="ts">
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
    import { submissionService } from "$services/submissions";
    import { authStore } from "$lib/stores/auth";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { LinkButton } from "$components";
    import type {
        Tournament,
        TournamentParticipant,
        Game,
        Submission,
    } from "$lib/models";
    const tournamentId = $derived(page.url.searchParams.get('id') || '');
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

    .loading-state, .error-state, .not-found {
        padding: var(--spacing-12);
        text-align: center;
        background-color: var(--color-bg-light);
    }

    .error-state {
        background-color: var(--color-bg-popup);
        color: var(--color-error);
    }

    .action-error {
        padding: var(--spacing-6);
        margin-bottom: var(--spacing-4);
        background-color: var(--color-bg-popup);
        color: var(--color-error);
    }

    .tournament-layout {
        display: grid;
        grid-template-columns: 2fr 1fr;
        gap: var(--spacing-6);
    }

    .content {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-4);
    }

    .tournament-details, .game-details, .participants-section, .submissions-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
    }

    .description {
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-4);
    }

    .details-grid {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: var(--spacing-2);
    }

    .details-grid dt {
        font-weight: 600;
        color: var(--color-fg-muted);
    }

    .languages-section {
        margin-bottom: var(--spacing-4);
    }

    .languages-section h3 {
        font-weight: 600;
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-2);
    }

    .language-tags {
        display: flex;
        gap: var(--spacing-2);
    }

    .empty-state {
        text-align: center;
        color: var(--color-fg-muted);
    }

    .participants-table th {
        border-bottom: 1px solid var(--color-border);
    }

    .participants-table td {
        border-bottom: 1px solid var(--color-border-light);
    }

    .submissions-table th {
        border-bottom: 1px solid var(--color-border);
    }

    .submissions-table td {
        border-bottom: 1px solid var(--color-border-light);
    }

    .rank {
        font-weight: 600;
    }

    .no-rank {
        color: var(--color-fg-muted);
    }

    .username {
        font-weight: 600;
    }

    .join-date, .submit-date {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
    }

    .language {
        font-weight: 600;
    }

    .notes {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-3);
    }

    .section-header h2 {
        font-size: 1.25rem;
        margin: 0;
    }

    .auth-message, .loading-message, .empty-message {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
    }

    .error-message {
        font-size: 0.875rem;
        color: var(--color-error);
    }

    .tournament-sidebar {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-4);
    }

    .actions-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
    }

    .actions-section h3 {
        margin-bottom: var(--spacing-4);
    }

    .auth-required {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-4);
    }

    .participant-status {
        margin-top: var(--spacing-4);
        padding: var(--spacing-4);
        background-color: var(--color-bg-popup);
        text-align: center;
    }

    .participant-status p {
        font-size: 0.875rem;
        font-weight: 600;
        color: var(--color-primary);
        margin: 0;
    }

    .status-info {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
    }

    .status-info h3 {
        margin-bottom: var(--spacing-2);
    }

    .status-info p {
        font-size: 0.875rem;
        margin: 0;
    }

    .registration-open {
        background-color: var(--color-bg-popup);
    }

    .registration-open h3, .registration-open p {
        color: var(--color-primary);
    }

    .tournament-running {
        background-color: var(--color-bg-popup);
    }

    .tournament-running h3, .tournament-running p {
        color: var(--color-warning);
    }

    .tournament-completed {
        background-color: var(--color-bg-popup);
    }

    .tournament-completed h3, .tournament-completed p {
        color: var(--color-success);
    }
</style>

<main>
    <div class="container">
        {#if loading}
            <section class="loading-state">
                <p>Loading tournament...</p>
            </section>
        {:else if error}
            <section class="error-state">
                <p>{error}</p>
            </section>
        {:else if tournament}
            <header>
                <div class="header-content">
                    <div class="title-section">
                        <h1>{tournament.name}</h1>
                        <span class={getStatusBadgeClass(tournament.status)}>
                            {tournament.status}
                        </span>
                    </div>
                    <LinkButton href="/tournaments" variant="secondary" label="Back to Tournaments" />
                </div>
            </header>

            {#if actionError}
                <section class="action-error">
                    <p>{actionError}</p>
                </section>
            {/if}

            <div class="tournament-layout">
                <div class="content">
                    <section class="tournament-details">
                        <h2>About This Tournament</h2>
                        <p class="description">{tournament.description}</p>
                        <dl class="details-grid">
                            <dt>Status:</dt>
                            <dd>
                                <span class={getStatusBadgeClass(tournament.status)}>
                                    {tournament.status}
                                </span>
                            </dd>
                            <dt>Players:</dt>
                            <dd>
                                {participants.length} / {tournament.max_players}
                                (min: {tournament.min_players})
                            </dd>
                            {#if tournament.start_time}
                                <dt>Start Time:</dt>
                                <dd>{formatDate(tournament.start_time)}</dd>
                            {/if}
                            {#if tournament.end_time}
                                <dt>End Time:</dt>
                                <dd>{formatDate(tournament.end_time)}</dd>
                            {/if}
                            <dt>Created:</dt>
                            <dd>{formatDate(tournament.created_at)}</dd>
                        </dl>
                    </section>

                    {#if game}
                        <section class="game-details">
                            <h2>Game: {game.name}</h2>
                            <p class="description">{game.description}</p>
                            <div class="languages-section">
                                <h3>Supported Languages:</h3>
                                <div class="language-tags">
                                    {#each game.supported_languages as lang}
                                        <span class="badge badge-scheduled">{lang}</span>
                                    {/each}
                                </div>
                            </div>
                        </section>
                    {/if}

                    <section class="participants-section">
                        <h2>Participants ({participants.length})</h2>
                        {#if participants.length === 0}
                            <p class="empty-state">No participants yet</p>
                        {:else}
                            <table class="participants-table">
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
                                                    <span class="rank">#{participant.rank}</span>
                                                {:else}
                                                    <span class="no-rank">-</span>
                                                {/if}
                                            </td>
                                            <td class="username">
                                                {participant.username || "Unknown"}
                                                {#if $authStore.user && participant.user_id === $authStore.user.id}
                                                    <span class="badge badge-registration">You</span>
                                                {/if}
                                            </td>
                                            <td>{participant.score || 0}</td>
                                            <td class="join-date">{formatDate(participant.joined_at)}</td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        {/if}
                    </section>

                    <section class="submissions-section">
                        <div class="section-header">
                            <h2>Your Submissions</h2>
                            {#if canSubmit()}
                                <LinkButton
                                    href="/tournament/submit?id={tournamentId}"
                                    variant="primary"
                                    label="New Submission"
                                />
                            {/if}
                        </div>
                        {#if !$authStore.isAuthenticated}
                            <p class="auth-message">Login to view and submit your code.</p>
                        {:else if submissionsLoading}
                            <p class="loading-message">Loading your submissions...</p>
                        {:else if submissionError}
                            <p class="error-message">{submissionError}</p>
                        {:else if userSubmissions.length === 0}
                            <p class="empty-message">You have not submitted any code yet.</p>
                        {:else}
                            <table class="submissions-table">
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
                                            <td class="language">{submission.language.toUpperCase()}</td>
                                            <td>
                                                <span class="badge badge-{submission.status}">
                                                    {submission.status}
                                                </span>
                                            </td>
                                            <td class="submit-date">{formatDate(submission.created_at)}</td>
                                            <td class="notes">{submission.error_message || "-"}</td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        {/if}
                    </section>
                </div>

                <aside class="tournament-sidebar">
                    <section class="actions-section">
                        <h3>Actions</h3>
                        {#if !$authStore.isAuthenticated}
                            <p class="auth-required">You must be logged in to participate</p>
                            <LinkButton href="/login" variant="primary" label="Login" />
                        {:else}
                            {#if canJoin()}
                                <button
                                    onclick={joinTournament}
                                    data-variant="success"
                                    disabled={actionLoading}
                                >
                                    {actionLoading ? "Joining..." : "Join Tournament"}
                                </button>
                            {/if}
                            {#if canLeave()}
                                <button
                                    onclick={leaveTournament}
                                    data-variant="danger"
                                    disabled={actionLoading}
                                >
                                    {actionLoading ? "Leaving..." : "Leave Tournament"}
                                </button>
                            {/if}
                            {#if canSubmit()}
                                <LinkButton
                                    href="/tournament/submit?id={tournamentId}"
                                    variant="primary"
                                    label="Submit Code"
                                />
                            {/if}
                            {#if isParticipant}
                                <div class="participant-status">
                                    <p>You are participating in this tournament</p>
                                </div>
                            {/if}
                        {/if}
                    </section>

                    {#if tournament.status === "registration"}
                        <section class="status-info registration-open">
                            <h3>Registration Open</h3>
                            <p>Join now to participate in this tournament!</p>
                        </section>
                    {:else if tournament.status === "running"}
                        <section class="status-info tournament-running">
                            <h3>Tournament In Progress</h3>
                            <p>This tournament is currently running. New participants cannot join.</p>
                        </section>
                    {:else if tournament.status === "completed"}
                        <section class="status-info tournament-completed">
                            <h3>Tournament Completed</h3>
                            <p>Check the participants list for final rankings.</p>
                        </section>
                    {/if}
                </aside>
            </div>
        {:else}
            <section class="not-found">
                <p>Tournament not found</p>
            </section>
        {/if}
    </div>
</main>
