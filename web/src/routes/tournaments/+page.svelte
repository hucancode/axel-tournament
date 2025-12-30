<script lang="ts">
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import { authStore } from "$lib/stores/auth";
    import { onMount } from "svelte";
    import type { Tournament, TournamentParticipant, Game } from "$lib/types";
    import { TournamentCard, LinkButton, LoadingCard, EmptyState } from "$lib/components";

    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
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

    const auth = $derived($authStore);
    const canManageTournaments = $derived(auth.isAuthenticated && (auth.user?.role === "admin" || auth.user?.role === "gamesetter"));

    onMount(async () => {
        await loadTournaments();
    });

    async function loadTournaments() {
        loading = true;
        error = "";
        try {
            const status = selectedStatus === "all" ? undefined : selectedStatus;
            const [tournamentsData, gamesData] = await Promise.all([
                tournamentService.list(status),
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
            participantCounts = participantResults.reduce(
                (acc, { tournamentId, participants }) => {
                    acc[tournamentId] = participants;
                    return acc;
                },
                {} as Record<string, TournamentParticipant[]>,
            );
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load tournaments";
            console.error("Failed to load tournaments:", err);
        } finally {
            loading = false;
        }
    }

    async function handleStatusChange() {
        await loadTournaments();
    }
</script>

<section class="container">
    <div class="flex justify-between items-center mb-4">
        <h1 class="text-2xl font-bold">Tournaments</h1>
        {#if canManageTournaments}
            <LinkButton href="/tournaments/new" label="+ Create Tournament" variant="primary" />
        {/if}
    </div>

    <div class="p-6 bg-hatch mb-4">
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
        <div
            class="p-6 bg-hatch bg-red-100 mb-4"
        >
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <LoadingCard message="Loading tournaments..." />
    {:else if tournaments.length === 0}
        <EmptyState message="No tournaments found" />
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
</section>
