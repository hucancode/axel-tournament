<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import type {
        Tournament,
        Game,
        CreateTournamentRequest,
        TournamentStatus,
    } from "$lib/types";
    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");
    // Create/Edit form state
    let showForm = $state(false);
    let editingTournament = $state<Tournament | null>(null);
    let formData = $state<CreateTournamentRequest>({
        game_id: "",
        name: "",
        description: "",
        status: "scheduled",
        min_players: 2,
        max_players: 100,
        start_time: "",
        end_time: "",
    });
    let formLoading = $state(false);
    let formError = $state("");
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
        if (formData.min_players < 1) {
            formError = "Minimum players must be at least 1";
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
        const submitData: CreateTournamentRequest = {
            ...formData,
            start_time: formData.start_time
                ? new Date(formData.start_time).toISOString()
                : undefined,
            end_time: formData.end_time
                ? new Date(formData.end_time).toISOString()
                : undefined,
        };
        try {
            if (isEditing && editingTournament) {
                await tournamentService.update(
                    editingTournament.id,
                    submitData,
                );
            } else {
                await tournamentService.create(submitData);
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
    {#if error}
        <div
            class="card"
            style="background-color: #fee2e2; border-left: 4px solid var(--red-600); margin-bottom: 1rem;"
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
            <div style="overflow-x: auto;">
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
                                        class="font-semibold"
                                        style="color: var(--primary-600); text-decoration: none;"
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
                                    {tournament.current_players} / {tournament.max_players}
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
                                    <button
                                        class="btn btn-secondary"
                                        style="padding: 0.25rem 0.75rem; font-size: 0.875rem;"
                                        onclick={() => openEditForm(tournament)}
                                    >
                                        Edit
                                    </button>
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
        style="
			position: fixed;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			background: rgba(0, 0, 0, 0.5);
			display: flex;
			align-items: center;
			justify-content: center;
			z-index: 1000;
			padding: 1rem;
		"
        onclick={(e) => {
            if (e.target === e.currentTarget) closeForm();
        }}
    >
        <div
            class="card"
            style="
				max-width: 700px;
				width: 100%;
				max-height: 90vh;
				overflow-y: auto;
			"
        >
            <h2 class="font-bold text-xl mb-4">
                {isEditing ? "Edit Tournament" : "Create New Tournament"}
            </h2>
            {#if formError}
                <div
                    style="background-color: #fee2e2; border-left: 4px solid var(--red-600); padding: 1rem; margin-bottom: 1rem;"
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
