<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import type {
        Tournament,
        TournamentParticipant,
        Game,
        CreateTournamentRequest,
        UpdateTournamentRequest,
        TournamentStatus,
        MatchGenerationType,
    } from "$lib/types";
    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
    let participantCounts = $state<Record<string, TournamentParticipant[]>>({});
    let loading = $state(true);
    let error = $state("");
    // Create/Edit form state
    let showForm = $state(false);
    let editingTournament = $state<Tournament | null>(null);
    type TournamentFormData = CreateTournamentRequest & {
        status: TournamentStatus;
        match_generation_type: MatchGenerationType;
    };
    let formData = $state<TournamentFormData>({
        game_id: "",
        name: "",
        description: "",
        status: "scheduled",
        min_players: 2,
        max_players: 100,
        start_time: "",
        end_time: "",
        match_generation_type: "all_vs_all",
    });
    let formLoading = $state(false);
    let formError = $state("");
    let actionError = $state("");
    let actionMessage = $state("");
    let actionLoading = $state<Record<string, boolean>>({});
    const auth = $derived($authStore);
    const isEditing = $derived(editingTournament !== null);
    onMount(async () => {
        // Check authentication and admin role
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        if (auth.user?.role !== "admin") {
            goto("/");
            return;
        }
        await loadData();
    });
    async function loadData() {
        loading = true;
        error = "";
        try {
            const [tournamentsData, gamesData] = await Promise.all([
                tournamentService.list(),
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
            participantCounts = participantResults.reduce((acc, { tournamentId, participants }) => {
                acc[tournamentId] = participants;
                return acc;
            }, {} as Record<string, TournamentParticipant[]>);
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load data";
        } finally {
            loading = false;
        }
    }
    function openCreateForm() {
        editingTournament = null;
        formData = {
            game_id: games.length > 0 ? games[0].id : "",
            name: "",
            description: "",
            status: "scheduled",
            min_players: 2,
            max_players: 100,
            start_time: "",
            end_time: "",
            match_generation_type: "all_vs_all",
        };
        formError = "";
        showForm = true;
    }
    function openEditForm(tournament: Tournament) {
        editingTournament = tournament;
        formData = {
            game_id: tournament.game_id,
            name: tournament.name,
            description: tournament.description,
            status: tournament.status,
            min_players: tournament.min_players,
            max_players: tournament.max_players,
            start_time: tournament.start_time
                ? formatDateForInput(tournament.start_time)
                : "",
            end_time: tournament.end_time
                ? formatDateForInput(tournament.end_time)
                : "",
            match_generation_type: tournament.match_generation_type,
        };
        formError = "";
        showForm = true;
    }
    function closeForm() {
        showForm = false;
        editingTournament = null;
        formError = "";
    }
    function formatDateForInput(dateStr: string): string {
        // Convert ISO string to datetime-local format (YYYY-MM-DDTHH:mm)
        return dateStr.slice(0, 16);
    }
    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";
        // Validation
        if (formData.min_players < 2) {
            formError = "Minimum players must be at least 2";
            formLoading = false;
            return;
        }
        if (formData.max_players < formData.min_players) {
            formError =
                "Maximum players must be greater than or equal to minimum players";
            formLoading = false;
            return;
        }
        // Prepare data with proper date formatting
        const createPayload: CreateTournamentRequest = {
            game_id: formData.game_id,
            name: formData.name,
            description: formData.description,
            min_players: formData.min_players,
            max_players: formData.max_players,
            start_time: formData.start_time
                ? new Date(formData.start_time).toISOString()
                : undefined,
            end_time: formData.end_time
                ? new Date(formData.end_time).toISOString()
                : undefined,
            match_generation_type: formData.match_generation_type,
        };
        const updatePayload: UpdateTournamentRequest = {
            name: formData.name,
            description: formData.description,
            status: formData.status,
            start_time: formData.start_time
                ? new Date(formData.start_time).toISOString()
                : undefined,
            end_time: formData.end_time
                ? new Date(formData.end_time).toISOString()
                : undefined,
        };
        try {
            if (isEditing && editingTournament) {
                await tournamentService.update(editingTournament.id, updatePayload);
            } else {
                const created = await tournamentService.create(createPayload);
                if (formData.status !== "scheduled") {
                    await tournamentService.update(created.id, {
                        status: formData.status,
                    });
                }
            }
            await loadData();
            closeForm();
        } catch (err) {
            formError =
                err instanceof Error
                    ? err.message
                    : "Failed to save tournament";
        } finally {
            formLoading = false;
        }
    }
    async function startTournament(tournamentId: string) {
        actionLoading = { ...actionLoading, [tournamentId]: true };
        actionError = "";
        actionMessage = "";
        try {
            await tournamentService.start(tournamentId);
            actionMessage = "Tournament started and matches were generated.";
            await loadData();
        } catch (err) {
            actionError =
                err instanceof Error
                    ? err.message
                    : "Failed to start tournament";
        } finally {
            actionLoading = { ...actionLoading, [tournamentId]: false };
        }
    }
    function formatDate(dateStr: string): string {
        return new Date(dateStr).toLocaleDateString("en-US", {
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }
    function getGameName(gameId: string): string {
        const game = games.find((g) => g.id === gameId);
        return game?.name || "Unknown Game";
    }
    function getStatusBadgeClass(status: TournamentStatus): string {
        const statusMap: Record<TournamentStatus, string> = {
            scheduled: "badge-scheduled",
            registration: "badge-registration",
            generating: "badge-generating",
            running: "badge-running",
            completed: "badge-completed",
            cancelled: "badge-cancelled",
        };
        return statusMap[status] || "badge-scheduled";
    }
</script>

<div class="container page">
    <div class="page-header">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="page-title">Tournament Management</h1>
                <p class="text-gray-500">Create and manage tournaments</p>
            </div>
            <div class="flex gap-2">
                <button class="btn btn-primary" onclick={openCreateForm}>
                    Create New Tournament
                </button>
                <a href="/admin" class="btn btn-secondary">Back to Dashboard</a>
            </div>
        </div>
    </div>
    {#if actionMessage}
        <div class="card bg-emerald-100 mb-4">
            <p class="text-green-700">{actionMessage}</p>
        </div>
    {/if}
    {#if actionError}
        <div class="card bg-red-100 mb-4">
            <p class="text-red-600">{actionError}</p>
        </div>
    {/if}
    {#if error}
        <div
            class="card bg-red-100 border-l-4 border-red-600 mb-4"
        >
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <div class="card text-center">
            <p class="text-gray-500">Loading tournaments...</p>
        </div>
    {:else if games.length === 0}
        <div class="card text-center">
            <p class="text-gray-500">
                Please create a game first before creating tournaments.
            </p>
            <a href="/admin/games" class="btn btn-primary mt-4">
                Go to Game Management
            </a>
        </div>
    {:else if tournaments.length === 0}
        <div class="card text-center">
            <p class="text-gray-500">
                No tournaments found. Create your first tournament!
            </p>
        </div>
    {:else}
        <div class="card">
            <div class="overflow-x-auto">
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Game</th>
                            <th>Status</th>
                            <th>Players</th>
                            <th>Start Time</th>
                            <th>End Time</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each tournaments as tournament}
                            <tr>
                                <td>
                                    <a
                                        href="/tournaments/{tournament.id}"
                                        class="font-semibold text-primary-600 no-underline"
                                    >
                                        {tournament.name}
                                    </a>
                                </td>
                                <td>{getGameName(tournament.game_id)}</td>
                                <td>
                                    <span
                                        class="badge {getStatusBadgeClass(
                                            tournament.status,
                                        )}"
                                    >
                                        {tournament.status}
                                    </span>
                                </td>
                                <td>
                                    {(participantCounts[tournament.id] || []).length} / {tournament.max_players}
                                    <span class="text-sm text-gray-500"
                                        >(min: {tournament.min_players})</span
                                    >
                                </td>
                                <td class="text-sm">
                                    {tournament.start_time
                                        ? formatDate(tournament.start_time)
                                        : "Not set"}
                                </td>
                                <td class="text-sm">
                                    {tournament.end_time
                                        ? formatDate(tournament.end_time)
                                        : "Not set"}
                                </td>
                                <td>
                                    <div
                                        class="flex gap-2 flex-wrap"
                                    >
                                        {#if tournament.status === "registration"}
                                            <button
                                                class="btn btn-success py-1 px-3 text-sm"
                                                onclick={() =>
                                                    startTournament(
                                                        tournament.id,
                                                    )}
                                                disabled={actionLoading[
                                                    tournament.id
                                                ]}
                                            >
                                                {actionLoading[tournament.id]
                                                    ? "Starting..."
                                                    : "Start"}
                                            </button>
                                        {/if}
                                        <button
                                            class="btn btn-secondary py-1 px-3 text-sm"
                                            onclick={() =>
                                                openEditForm(tournament)}
                                        >
                                            Edit
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}
</div>

<!-- Create/Edit Form Modal -->
{#if showForm}
    <div
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000] p-4"
        role="button"
        tabindex="0"
        aria-label="Close tournament form"
        onclick={(e) => {
            if (e.target === e.currentTarget) closeForm();
        }}
        onkeydown={(e) => {
            if (
                e.target === e.currentTarget &&
                (e.key === "Escape" || e.key === "Enter" || e.key === " ")
            ) {
                e.preventDefault();
                closeForm();
            }
        }}
    >
        <div
            class="card max-w-[700px] w-full max-h-[90vh] overflow-y-auto"
        >
            <h2 class="font-bold text-xl mb-4">
                {isEditing ? "Edit Tournament" : "Create New Tournament"}
            </h2>
            {#if formError}
                <div
                    class="bg-red-100 border-l-4 border-red-600 p-4 mb-4"
                >
                    <p class="text-red-600">{formError}</p>
                </div>
            {/if}
            <form onsubmit={handleSubmit}>
                <div class="form-group">
                    <label for="game_id">Game</label>
                    <select
                        id="game_id"
                        class="select"
                        bind:value={formData.game_id}
                        disabled={formLoading}
                        required
                    >
                        {#each games as game}
                            <option value={game.id}>{game.name}</option>
                        {/each}
                    </select>
                </div>
                <div class="form-group">
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
                <div class="form-group">
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
                <div class="form-group">
                    <label for="match_generation_type"
                        >Match Generation Type</label
                    >
                    <select
                        id="match_generation_type"
                        class="select"
                        bind:value={formData.match_generation_type}
                        disabled={formLoading || isEditing}
                        required
                    >
                        <option value="all_vs_all">All vs All</option>
                        <option value="round_robin">Round Robin</option>
                        <option value="single_elimination">
                            Single Elimination
                        </option>
                        <option value="double_elimination">
                            Double Elimination
                        </option>
                    </select>
                    {#if isEditing}
                        <p
                            class="text-sm text-gray-500 mt-1"
                        >
                            Match generation type cannot be changed after
                            creation.
                        </p>
                    {/if}
                </div>
                <div class="form-group">
                    <label for="status">Status</label>
                    <select
                        id="status"
                        class="select"
                        bind:value={formData.status}
                        disabled={formLoading}
                        required
                    >
                        <option value="scheduled">Scheduled</option>
                        <option value="registration">Registration</option>
                        <option value="generating">Generating</option>
                        <option value="running">Running</option>
                        <option value="completed">Completed</option>
                        <option value="cancelled">Cancelled</option>
                    </select>
                </div>
                <div class="grid grid-2 gap-4">
                    <div class="form-group">
                        <label for="min_players">Minimum Players</label>
                        <input
                            id="min_players"
                            type="number"
                            class="input"
                            bind:value={formData.min_players}
                            disabled={formLoading}
                            min="1"
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label for="max_players">Maximum Players</label>
                        <input
                            id="max_players"
                            type="number"
                            class="input"
                            bind:value={formData.max_players}
                            disabled={formLoading}
                            min="1"
                            required
                        />
                    </div>
                </div>
                <div class="grid grid-2 gap-4">
                    <div class="form-group">
                        <label for="start_time">Start Time (Optional)</label>
                        <input
                            id="start_time"
                            type="datetime-local"
                            class="input"
                            bind:value={formData.start_time}
                            disabled={formLoading}
                        />
                    </div>
                    <div class="form-group">
                        <label for="end_time">End Time (Optional)</label>
                        <input
                            id="end_time"
                            type="datetime-local"
                            class="input"
                            bind:value={formData.end_time}
                            disabled={formLoading}
                        />
                    </div>
                </div>
                <div class="flex gap-2">
                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled={formLoading}
                    >
                        {formLoading
                            ? "Saving..."
                            : isEditing
                              ? "Update Tournament"
                              : "Create Tournament"}
                    </button>
                    <button
                        type="button"
                        class="btn btn-secondary"
                        disabled={formLoading}
                        onclick={closeForm}
                    >
                        Cancel
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}
