<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { tournamentService } from "$lib/services/tournaments";
    import { gameService } from "$lib/services/games";
    import { Button, LinkButton, LoadingCard, EmptyState } from "$lib/components";
    import type { Game, CreateTournamentRequest, MatchGenerationType } from "$lib/types";

    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");

    let formData = $state<CreateTournamentRequest>({
        game_id: "",
        name: "",
        description: "",
        min_players: 2,
        max_players: 100,
        start_time: "",
        end_time: "",
        match_generation_type: "all_vs_all",
    });
    let formLoading = $state(false);
    let formError = $state("");

    const auth = $derived($authStore);
    const matchTypes: { value: MatchGenerationType; label: string }[] = [
        { value: "all_vs_all", label: "All vs All" },
        { value: "round_robin", label: "Round Robin" },
        { value: "single_elimination", label: "Single Elimination" },
        { value: "double_elimination", label: "Double Elimination" },
    ];

    // Get games owned by the current user
    const ownedGames = $derived(() => {
        if (!auth.isAuthenticated) return [];
        if (auth.user?.role === "admin") return games;
        if (auth.user?.role === "gamesetter") {
            return games.filter(g => g.owner_id === auth.user?.id);
        }
        return [];
    });

    onMount(async () => {
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        if (auth.user?.role !== "admin" && auth.user?.role !== "gamesetter") {
            goto("/tournaments");
            return;
        }
        await loadGames();
    });

    async function loadGames() {
        loading = true;
        error = "";
        try {
            games = await gameService.list();
            const owned = ownedGames();
            if (owned.length > 0) {
                formData.game_id = owned[0].id;
            }
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load games";
        } finally {
            loading = false;
        }
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";

        if (formData.min_players < 2) {
            formError = "Minimum players must be at least 2";
            formLoading = false;
            return;
        }

        if (formData.max_players < formData.min_players) {
            formError = "Maximum players must be greater than or equal to minimum players";
            formLoading = false;
            return;
        }

        try {
            await tournamentService.create(formData);
            goto("/tournaments");
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to create tournament";
        } finally {
            formLoading = false;
        }
    }
</script>

<section class="container">
    <div class="page-header">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="page-title">Create New Tournament</h1>
                <p class="text-gray-500">Set up a new competitive tournament</p>
            </div>
            <LinkButton variant="secondary" href="/tournaments" label="Back to Tournaments" />
        </div>
    </div>

    {#if loading}
        <LoadingCard message="Loading games..." />
    {:else if error}
        <div class="bg-red-100 border-l-4 border-red-600 p-4 mb-4">
            <p class="text-red-600">{error}</p>
        </div>
    {:else if ownedGames().length === 0}
        <EmptyState
            message="You need to create a game before creating tournaments."
            actionLabel="Create Game"
            onAction={() => goto("/games/new")}
        />
    {:else}
        {#if formError}
            <div class="bg-red-100 border-l-4 border-red-600 p-4 mb-4">
                <p class="text-red-600">{formError}</p>
            </div>
        {/if}

        <form onsubmit={handleSubmit} class="border border-[--border-color] p-6 shadow-sm bg-hatch">
            <div class="mb-4">
                <label for="game" class="block mb-2 font-medium text-gray-dark">Game</label>
                <select
                    id="game"
                    class="input"
                    bind:value={formData.game_id}
                    disabled={formLoading}
                    required
                >
                    {#each ownedGames() as game}
                        <option value={game.id}>{game.name}</option>
                    {/each}
                </select>
            </div>

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

            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
                    <label for="min-players" class="block mb-2 font-medium text-gray-dark">Minimum Players</label>
                    <input
                        id="min-players"
                        type="number"
                        class="input"
                        min="2"
                        bind:value={formData.min_players}
                        disabled={formLoading}
                        required
                    />
                </div>

                <div>
                    <label for="max-players" class="block mb-2 font-medium text-gray-dark">Maximum Players</label>
                    <input
                        id="max-players"
                        type="number"
                        class="input"
                        min="2"
                        bind:value={formData.max_players}
                        disabled={formLoading}
                        required
                    />
                </div>
            </div>

            <div class="mb-4">
                <label for="match-type" class="block mb-2 font-medium text-gray-dark">Match Generation Type</label>
                <select
                    id="match-type"
                    class="input"
                    bind:value={formData.match_generation_type}
                    disabled={formLoading}
                    required
                >
                    {#each matchTypes as type}
                        <option value={type.value}>{type.label}</option>
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
                    label={formLoading ? "Creating..." : "Create Tournament"}
                />
                <LinkButton
                    href="/tournaments"
                    variant="secondary"
                    label="Cancel"
                />
            </div>
        </form>
    {/if}
</section>
