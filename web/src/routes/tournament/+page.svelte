<script lang="ts">
    import { tournamentService } from "$services/tournaments";
    import { gameService } from "$services/games";
    import { submissionService } from "$services/submissions";
    import { matchService } from "$services/matches";
    import { roomService } from "$services/rooms";
    import { authStore } from "$lib/stores/auth";
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { LinkButton, BracketView } from "$components";
    import type {
        Tournament,
        TournamentParticipant,
        Game,
        Match,
        Submission,
        SubmissionStats,
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
    let myParticipant = $state<TournamentParticipant | null>(null);
    let submissionStats = $state<Record<string, SubmissionStats>>({});
    let queueLoading = $state(false);
    let matches = $state<Match[]>([]);
    let matchesLoading = $state(false);
    let detailMatch = $state<Match | null>(null);
    let detailDialog: HTMLDialogElement | null = $state(null);
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
                myParticipant = participants.find(
                    (p) => p.user_id === $authStore.user!.id,
                ) ?? null;
                isParticipant = myParticipant !== null;
                await loadUserSubmissions();
            } else {
                userSubmissions = [];
                myParticipant = null;
            }
            // Load game information
            if (tournament.game_id) {
                game = await gameService.get(tournament.game_id);
            }
            await loadMatches();
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
        submissionStats = {};
        try {
            userSubmissions = await submissionService.list(tournamentId);
            // Pull stats per bot in parallel; ignore individual failures.
            const entries = await Promise.all(
                userSubmissions.map(async (s) => {
                    try {
                        const stats = await submissionService.stats(s.id);
                        return [s.id, stats] as const;
                    } catch {
                        return [s.id, null] as const;
                    }
                }),
            );
            submissionStats = Object.fromEntries(
                entries.filter(([, v]) => v !== null) as [string, SubmissionStats][],
            );
        } catch (err) {
            submissionError =
                err instanceof Error
                    ? err.message
                    : "Failed to load your submissions";
        } finally {
            submissionsLoading = false;
        }
    }

    async function loadMatches() {
        matchesLoading = true;
        try {
            matches = await matchService.list({ tournament_id: tournamentId });
        } catch {
            matches = [];
        } finally {
            matchesLoading = false;
        }
    }

    function usernameFor(userId: string): string {
        return participants.find((p) => p.user_id === userId)?.username ?? userId;
    }

    function openDetail(m: Match) {
        detailMatch = m;
        detailDialog?.showModal();
    }

    function closeDetail() {
        detailDialog?.close();
        detailMatch = null;
    }

    function fmtDate(s?: string): string {
        if (!s) return "-";
        return new Date(s).toLocaleString();
    }

    function faultLabel(m: Match): string {
        if (m.faulted_user_ids.length === 0) return "";
        const names = m.faulted_user_ids.map(usernameFor).join(", ");
        const reason = m.error_message ?? "fault";
        return `${names}: ${reason}`;
    }

    async function selectBot(submissionId: string) {
        actionLoading = true;
        actionError = "";
        try {
            await submissionService.select(submissionId);
            await loadTournamentData();
        } catch (err) {
            actionError =
                err instanceof Error ? err.message : "Failed to select bot";
        } finally {
            actionLoading = false;
        }
    }

    async function enqueueRanked() {
        queueLoading = true;
        actionError = "";
        try {
            await roomService.enqueueRanked(tournamentId);
        } catch (err) {
            actionError =
                err instanceof Error ? err.message : "Failed to join queue";
        } finally {
            queueLoading = false;
        }
    }

    async function dequeueRanked() {
        queueLoading = true;
        actionError = "";
        try {
            await roomService.dequeueRanked(tournamentId);
        } catch (err) {
            actionError =
                err instanceof Error ? err.message : "Failed to leave queue";
        } finally {
            queueLoading = false;
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

    .active-bot-row {
        background-color: var(--color-bg-popup);
    }

    .record {
        font-variant-numeric: tabular-nums;
    }

    .ranked-queue {
        margin-top: var(--spacing-4);
        padding: var(--spacing-4);
        background-color: var(--color-bg-popup);
    }

    .ranked-queue h4 {
        margin: 0 0 var(--spacing-2) 0;
    }

    .ranked-queue p {
        font-size: 0.875rem;
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-3);
    }

    .queue-buttons {
        display: flex;
        gap: var(--spacing-2);
    }

    .badge-bot {
        background-color: var(--color-info);
        color: var(--color-bg);
    }

    .badge-human {
        background-color: var(--color-warning);
        color: var(--color-bg);
    }

    .matches-section,
    .bracket-section {
        padding: var(--spacing-6);
        background-color: var(--color-bg-light);
    }

    .matches-table th {
        border-bottom: 1px solid var(--color-border);
    }

    .matches-table td {
        border-bottom: 1px solid var(--color-border-light);
    }

    .badge-fault {
        background-color: var(--color-error);
        color: var(--color-bg);
        cursor: help;
    }

    .fault-cell {
        font-size: 0.875rem;
    }

    .match-row {
        cursor: pointer;
    }

    .match-row:hover {
        background-color: var(--color-bg-popup);
    }

    .match-detail {
        min-width: 24rem;
        max-width: 36rem;
        padding: var(--spacing-4);
        background-color: var(--color-bg-light);
        border: 1px solid var(--color-border);
    }

    .match-detail header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-3);
    }

    .match-detail dl {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: var(--spacing-2);
        margin-bottom: var(--spacing-3);
    }

    .match-detail dt {
        font-weight: 600;
        color: var(--color-fg-muted);
    }

    .error-text {
        color: var(--color-error);
    }

    .part-list {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: var(--spacing-2);
    }

    .part-list li {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: var(--spacing-2);
        background-color: var(--color-bg);
        border-left: 3px solid transparent;
    }

    .part-list li.faulted {
        border-left-color: var(--color-error);
    }

    .replay-link {
        margin-top: var(--spacing-4);
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
                            <dt>Kind:</dt>
                            <dd>
                                <span class="badge badge-{tournament.kind}">
                                    {tournament.kind === "human" ? "Human vs Human" : "Bot vs Bot"}
                                </span>
                            </dd>
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
                                        <th>W / L / D</th>
                                        <th>Score</th>
                                        {#if tournament.kind === "human"}
                                            <th>ELO</th>
                                        {/if}
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
                                            <td>
                                                {participant.wins ?? 0} / {participant.losses ?? 0} / {participant.draws ?? 0}
                                            </td>
                                            <td>{(participant.score ?? 0).toFixed(1)}</td>
                                            {#if tournament.kind === "human"}
                                                <td>{participant.elo?.toFixed(0) ?? "1000"}</td>
                                            {/if}
                                            <td class="join-date">{formatDate(participant.joined_at)}</td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        {/if}
                    </section>

                    {#if tournament.match_generation_type === "single_elimination" || tournament.match_generation_type === "double_elimination"}
                        <section class="bracket-section">
                            <h2>Bracket</h2>
                            <BracketView {matches} {usernameFor} />
                        </section>
                    {/if}

                    <section class="matches-section">
                        <h2>Matches ({matches.length})</h2>
                        {#if matchesLoading}
                            <p class="loading-message">Loading matches...</p>
                        {:else if matches.length === 0}
                            <p class="empty-state">No matches yet.</p>
                        {:else}
                            <table class="matches-table">
                                <thead>
                                    <tr>
                                        <th>Round</th>
                                        <th>Players</th>
                                        <th>Score</th>
                                        <th>Status</th>
                                        <th>Fault</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each matches as m}
                                        {@const fault = faultLabel(m)}
                                        <tr class="match-row" onclick={() => openDetail(m)}>
                                            <td>
                                                {#if m.round !== null && m.round !== undefined}
                                                    R{m.round}{m.bracket ? ` ${m.bracket}` : ""}
                                                {:else}
                                                    -
                                                {/if}
                                            </td>
                                            <td>
                                                {m.participants
                                                    .map((p) => usernameFor(p.user_id ?? ""))
                                                    .join(" vs ")}
                                            </td>
                                            <td class="record">
                                                {m.participants
                                                    .map((p) =>
                                                        p.score === null || p.score === undefined
                                                            ? "-"
                                                            : p.score.toFixed(1),
                                                    )
                                                    .join(" / ")}
                                            </td>
                                            <td>
                                                <span class="badge badge-{m.status}">{m.status}</span>
                                            </td>
                                            <td class="fault-cell">
                                                {#if fault}
                                                    <span class="badge badge-fault" title={fault}>
                                                        ⚠ {fault}
                                                    </span>
                                                {:else}
                                                    -
                                                {/if}
                                            </td>
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
                                        <th>W / L / D</th>
                                        <th>Score</th>
                                        <th>Submitted</th>
                                        <th>Active</th>
                                        <th>Notes</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each userSubmissions as submission, index}
                                        {@const stats = submissionStats[submission.id]}
                                        {@const isActive = myParticipant?.submission_id === submission.id}
                                        <tr class={isActive ? "active-bot-row" : ""}>
                                            <td>#{index + 1}</td>
                                            <td class="language">{submission.language.toUpperCase()}</td>
                                            <td>
                                                <span class="badge badge-{submission.status}">
                                                    {submission.status}
                                                </span>
                                            </td>
                                            <td class="record">
                                                {#if stats}
                                                    {stats.wins} / {stats.losses} / {stats.draws}
                                                {:else}
                                                    -
                                                {/if}
                                            </td>
                                            <td>{stats?.total_score?.toFixed(1) ?? "-"}</td>
                                            <td class="submit-date">{formatDate(submission.created_at)}</td>
                                            <td>
                                                {#if isActive}
                                                    <span class="badge badge-running">Active</span>
                                                {:else if submission.status === "accepted" && tournament.status === "registration"}
                                                    <button
                                                        data-variant="primary"
                                                        onclick={() => selectBot(submission.id)}
                                                        disabled={actionLoading}
                                                    >
                                                        Select
                                                    </button>
                                                {:else}
                                                    -
                                                {/if}
                                            </td>
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
                            {#if isParticipant && tournament.kind === "human" && tournament.status === "running"}
                                <div class="ranked-queue">
                                    <h4>Ranked Match Queue</h4>
                                    <p>Get auto-paired with another player. The system creates a private room when a partner is found.</p>
                                    <div class="queue-buttons">
                                        <button
                                            data-variant="primary"
                                            onclick={enqueueRanked}
                                            disabled={queueLoading}
                                        >
                                            {queueLoading ? "..." : "Find Match"}
                                        </button>
                                        <button
                                            data-variant="secondary"
                                            onclick={dequeueRanked}
                                            disabled={queueLoading}
                                        >
                                            Leave Queue
                                        </button>
                                    </div>
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

<dialog bind:this={detailDialog} class="match-detail">
    {#if detailMatch}
        {@const m = detailMatch}
        <header>
            <h2>Match Detail</h2>
            <button type="button" onclick={closeDetail} aria-label="Close">×</button>
        </header>
        <section class="detail-body">
            <dl>
                <dt>Status</dt>
                <dd><span class="badge badge-{m.status}">{m.status}</span></dd>
                {#if m.round !== null && m.round !== undefined}
                    <dt>Round</dt>
                    <dd>R{m.round}{m.bracket ? ` ${m.bracket}` : ""}</dd>
                {/if}
                <dt>Started</dt>
                <dd>{fmtDate(m.started_at)}</dd>
                <dt>Completed</dt>
                <dd>{fmtDate(m.completed_at)}</dd>
                {#if m.error_message}
                    <dt>Error</dt>
                    <dd class="error-text">{m.error_message}</dd>
                {/if}
            </dl>
            <h3>Participants</h3>
            <ul class="part-list">
                {#each m.participants as p}
                    {@const isFaulted = p.user_id && m.faulted_user_ids.includes(p.user_id)}
                    <li class:faulted={isFaulted}>
                        <span class="name">{usernameFor(p.user_id ?? "")}</span>
                        <span class="score">
                            {p.score === null || p.score === undefined ? "-" : p.score.toFixed(2)}
                        </span>
                        {#if isFaulted}
                            <span class="badge badge-fault">faulted</span>
                        {/if}
                    </li>
                {/each}
            </ul>
            {#if m.room_id}
                <p class="replay-link">
                    <a href={`/room?id=${encodeURIComponent(m.room_id)}&game=${encodeURIComponent(m.game_id)}`}>
                        Open replay →
                    </a>
                </p>
            {/if}
        </section>
    {/if}
</dialog>
