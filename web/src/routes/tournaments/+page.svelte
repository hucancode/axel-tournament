<script lang="ts">
    import { tournamentService } from "$lib/services/tournaments";
    import { onMount } from "svelte";
    import type { Tournament } from "$lib/types";
    let tournaments = $state<Tournament[]>([]);
    let loading = $state(true);
    let error = $state("");
    let selectedStatus = $state<string>("all");
    const statusOptions = [
        { value: "all", label: "All Tournaments" },
        { value: "scheduled", label: "Scheduled" },
        { value: "registration", label: "Registration Open" },
        { value: "running", label: "Running" },
        { value: "completed", label: "Completed" },
    ];
    onMount(async () => {
        await loadTournaments();
    });
    async function loadTournaments() {
        loading = true;
        error = "";
        try {
            const status =
                selectedStatus === "all" ? undefined : selectedStatus;
            tournaments = await tournamentService.list(status);
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load tournaments";
            console.error("Failed to load tournaments:", err);
        } finally {
            loading = false;
        }
    }
    async function handleStatusChange() {
        await loadTournaments();
    }
    function getStatusBadgeClass(status: string): string {
        return `badge badge-${status}`;
    }
    function formatDate(dateStr?: string): string {
        if (!dateStr) return "Not set";
        return new Date(dateStr).toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
            year: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }
</script>

<div class="page">
    <div class="container">
        <div class="page-header">
            <h1 class="page-title">Tournaments</h1>
            <p class="text-gray-500">Browse and join programming tournaments</p>
        </div>
        <div class="card mb-4">
            <div class="flex items-center gap-4">
                <label for="status-filter" class="font-semibold"
                    >Filter by Status:</label
                >
                <select
                    id="status-filter"
                    class="select"
                    style="width: auto;"
                    bind:value={selectedStatus}
                    onchange={handleStatusChange}
                    disabled={loading}
                >
                    {#each statusOptions as option}
                        <option value={option.value}>{option.label}</option>
                    {/each}
                </select>
            </div>
        </div>
        {#if error}
            <div class="card" style="background: #fee2e2; margin-bottom: 1rem;">
                <p class="text-red-600">{error}</p>
            </div>
        {/if}
        {#if loading}
            <div class="card text-center">
                <p class="text-gray-500">Loading tournaments...</p>
            </div>
        {:else if tournaments.length === 0}
            <div class="card text-center">
                <p class="text-gray-500">No tournaments found</p>
            </div>
        {:else}
            <div class="grid grid-2">
                {#each tournaments as tournament}
                    <div class="card">
                        <div class="flex justify-between items-center mb-2">
                            <h2 class="text-xl font-semibold">
                                {tournament.name}
                            </h2>
                            <span
                                class={getStatusBadgeClass(tournament.status)}
                            >
                                {tournament.status}
                            </span>
                        </div>
                        <p class="text-gray-700 mb-4">
                            {tournament.description}
                        </p>
                        <div
                            class="grid gap-2 mb-4"
                            style="grid-template-columns: auto 1fr;"
                        >
                            <div class="text-sm text-gray-500 font-semibold">
                                Players:
                            </div>
                            <div class="text-sm">
                                {tournament.current_players} / {tournament.max_players}
                                {#if tournament.current_players >= tournament.max_players}
                                    <span
                                        class="badge badge-failed"
                                        style="margin-left: 0.5rem;">Full</span
                                    >
                                {:else if tournament.current_players >= tournament.min_players}
                                    <span
                                        class="badge badge-accepted"
                                        style="margin-left: 0.5rem;"
                                        >Active</span
                                    >
                                {:else}
                                    <span
                                        class="badge badge-pending"
                                        style="margin-left: 0.5rem;"
                                    >
                                        Need {tournament.min_players -
                                            tournament.current_players} more
                                    </span>
                                {/if}
                            </div>
                            {#if tournament.start_time}
                                <div
                                    class="text-sm text-gray-500 font-semibold"
                                >
                                    Start:
                                </div>
                                <div class="text-sm">
                                    {formatDate(tournament.start_time)}
                                </div>
                            {/if}
                            {#if tournament.end_time}
                                <div
                                    class="text-sm text-gray-500 font-semibold"
                                >
                                    End:
                                </div>
                                <div class="text-sm">
                                    {formatDate(tournament.end_time)}
                                </div>
                            {/if}
                        </div>
                        <a
                            href="/tournaments/{tournament.id}"
                            class="btn btn-primary"
                            style="width: 100%;"
                        >
                            View Details
                        </a>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>
