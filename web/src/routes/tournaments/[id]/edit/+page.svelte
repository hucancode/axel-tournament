<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import { Button, LinkButton, LoadingCard } from "$lib/components";
    import type { Tournament, Game, UpdateTournamentRequest, TournamentStatus } from "$lib/types";

    let tournamentId = $derived($page.params.id);
    let tournament = $state<Tournament | null>(null);
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");

    let formData = $state<UpdateTournamentRequest>({
        name: "",
        description: "",
        status: "scheduled",
        start_time: "",
        end_time: "",
    });
    let formLoading = $state(false);
    let formError = $state("");

    const auth = $derived($authStore);
    const statusOptions: { value: TournamentStatus; label: string }[] = [
        { value: "scheduled", label: "Scheduled" },
        { value: "registration", label: "Registration Open" },
        { value: "generating", label: "Generating Matches" },
        { value: "running", label: "Running" },
        { value: "completed", label: "Completed" },
        { value: "cancelled", label: "Cancelled" },
    ];

    onMount(async () => {
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        await loadData();
    });

    async function loadData() {
        loading = true;
        error = "";
        try {
            const [tournamentData, gamesData] = await Promise.all([
                tournamentService.get(tournamentId),
                gameService.list(),
            ]);
            tournament = tournamentData;
            games = gamesData;

            // Check permissions
            const game = games.find(g => g.id === tournament!.game_id);
            if (auth.user?.role !== "admin" && game?.owner_id !== auth.user?.id) {
                goto("/tournaments");
                return;
            }

            formData = {
                name: tournament.name,
                description: tournament.description,
                status: tournament.status,
                start_time: tournament.start_time ? formatDateForInput(tournament.start_time) : "",
                end_time: tournament.end_time ? formatDateForInput(tournament.end_time) : "",
            };
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load tournament";
        } finally {
            loading = false;
        }
    }

    function formatDateForInput(dateStr: string): string {
        return dateStr.slice(0, 16);
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";

        try {
            await tournamentService.update(tournamentId, formData);
            goto(`/tournaments/${tournamentId}`);
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to update tournament";
        } finally {
            formLoading = false;
        }
    }

    async function handleStartTournament() {
        formLoading = true;
        formError = "";
        try {
            await tournamentService.start(tournamentId);
            goto(`/tournaments/${tournamentId}`);
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to start tournament";
        } finally {
            formLoading = false;
        }
    }
</script>

<div class="container page">
    <div class="page-header">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="page-title">Edit Tournament</h1>
                <p class="text-gray-500">Update tournament settings</p>
            </div>
            <LinkButton variant="secondary" href="/tournaments/{tournamentId}" label="Back to Tournament" />
        </div>
    </div>

    {#if loading}
        <LoadingCard message="Loading tournament..." />
    {:else if error}
        <div class="bg-red-100 border-l-4 border-red-600 p-4 mb-4">
            <p class="text-red-600">{error}</p>
        </div>
    {:else if tournament}
        {#if formError}
            <div class="bg-red-100 border-l-4 border-red-600 p-4 mb-4">
                <p class="text-red-600">{formError}</p>
            </div>
        {/if}

        <form onsubmit={handleSubmit} class="border border-[--border-color] p-6 shadow-sm bg-hatch">
            <div class="mb-4">
                <label for="name" class="block mb-2 font-medium text-gray-dark">Tournament Name</label>
                <input
                    id="name"
                    type="text"
                    class="input"
                    bind:value={formData.name}
                    disabled={formLoading}
                    required
                />
            </div>

            <div class="mb-4">
                <label for="description" class="block mb-2 font-medium text-gray-dark">Description</label>
                <textarea
                    id="description"
                    class="textarea"
                    bind:value={formData.description}
                    disabled={formLoading}
                    rows="4"
                    required
                ></textarea>
            </div>

            <div class="mb-4">
                <label for="status" class="block mb-2 font-medium text-gray-dark">Status</label>
                <select
                    id="status"
                    class="input"
                    bind:value={formData.status}
                    disabled={formLoading}
                    required
                >
                    {#each statusOptions as option}
                        <option value={option.value}>{option.label}</option>
                    {/each}
                </select>
            </div>

            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
                    <label for="start-time" class="block mb-2 font-medium text-gray-dark">Start Time (Optional)</label>
                    <input
                        id="start-time"
                        type="datetime-local"
                        class="input"
                        bind:value={formData.start_time}
                        disabled={formLoading}
                    />
                </div>

                <div>
                    <label for="end-time" class="block mb-2 font-medium text-gray-dark">End Time (Optional)</label>
                    <input
                        id="end-time"
                        type="datetime-local"
                        class="input"
                        bind:value={formData.end_time}
                        disabled={formLoading}
                    />
                </div>
            </div>

            <div class="flex gap-2">
                <Button
                    type="submit"
                    variant="primary"
                    disabled={formLoading}
                    label={formLoading ? "Updating..." : "Update Tournament"}
                />
                {#if tournament.status === "registration"}
                    <Button
                        type="button"
                        variant="success"
                        disabled={formLoading}
                        label="Start Tournament"
                        onclick={handleStartTournament}
                    />
                {/if}
                <LinkButton
                    href="/tournaments/{tournamentId}"
                    variant="secondary"
                    label="Cancel"
                />
            </div>
        </form>
    {/if}
</div>
