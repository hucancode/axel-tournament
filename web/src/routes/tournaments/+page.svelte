<script lang="ts">
    import { tournamentService } from "$lib/services/tournaments";
    import { onMount } from "svelte";
    import type { Tournament, TournamentParticipant } from "$lib/types";
    import { TournamentCard } from "$lib/components";
    let tournaments = $state<Tournament[]>([]);
    let participantCounts = $state<Record<string, TournamentParticipant[]>>({});
    let loading = $state(true);
    let error = $state("");
    let selectedStatus = $state<string>("all");
    const statusOptions = [
        { value: "all", label: "All Tournaments" },
        { value: "scheduled", label: "Scheduled" },
        { value: "registration", label: "Registration Open" },
        { value: "generating", label: "Generating Matches" },
        { value: "running", label: "Running" },
        { value: "completed", label: "Completed" },
        { value: "cancelled", label: "Cancelled" },
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
                    class="select w-auto"
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
            <div class="card bg-red-100 mb-4">
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
                    <TournamentCard
                        {tournament}
                        participants={participantCounts[tournament.id] || []}
                    />
                {/each}
            </div>
        {/if}
    </div>
</div>
