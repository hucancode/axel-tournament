<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { gameService } from "$lib/services/games";
    import { Button, LinkButton } from "$lib/components";
    import type {
        Game,
        ProgrammingLanguage,
        CreateGameRequest,
        UpdateGameRequest,
    } from "$lib/types";
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");
    // Create/Edit form state
    let showForm = $state(false);
    let editingGame = $state<Game | null>(null);
    type GameFormData = CreateGameRequest & { is_active: boolean };

    let formData = $state<GameFormData>({
        name: "",
        description: "",
        game_type: "automated",
        supported_languages: [],
        game_code: "",
        game_language: "rust",
        rounds_per_match: 3,
        repetitions: 1,
        timeout_ms: 5000,
        cpu_limit: 1.0,
        memory_limit_mb: 2,
        turn_timeout_ms: 200,
        is_active: true,
    });
    let formLoading = $state(false);
    let formError = $state("");
    const auth = $derived($authStore);
    const isEditing = $derived(editingGame !== null);
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
        await loadGames();
    });
    async function loadGames() {
        loading = true;
        error = "";
        try {
            games = await gameService.list();
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load games";
        } finally {
            loading = false;
        }
    }
    function openCreateForm() {
        editingGame = null;
        formData = {
            name: "",
            description: "",
            game_type: "automated",
            supported_languages: [],
            game_code: "",
            game_language: "rust",
            rounds_per_match: 3,
            repetitions: 1,
            timeout_ms: 120,
            cpu_limit: 1.0,
            memory_limit_mb: 512,
            turn_timeout_ms: 200,
            is_active: true,
        };
        formError = "";
        showForm = true;
    }
    function openEditForm(game: Game) {
        editingGame = game;
        formData = {
            name: game.name,
            description: game.description,
            game_type: game.game_type,
            supported_languages: [...game.supported_languages],
            game_code: game.game_code,
            game_language: game.game_language,
            rounds_per_match: game.rounds_per_match,
            repetitions: game.repetitions,
            timeout_ms: game.timeout_ms,
            cpu_limit: game.cpu_limit,
            memory_limit_mb: game.memory_limit_mb,
            turn_timeout_ms: game.turn_timeout_ms,
            is_active: game.is_active,
        };
        formError = "";
        showForm = true;
    }
    function closeForm() {
        showForm = false;
        editingGame = null;
        formError = "";
    }
    function toggleLanguage(lang: ProgrammingLanguage) {
        if (formData.supported_languages.includes(lang)) {
            formData.supported_languages = formData.supported_languages.filter(
                (l) => l !== lang,
            );
        } else {
            formData.supported_languages = [
                ...formData.supported_languages,
                lang,
            ];
        }
    }
    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";
        if (formData.supported_languages.length === 0) {
            formError = "Please select at least one supported language";
            formLoading = false;
            return;
        }
        if (!formData.game_code.trim()) {
            formError = "Game code is required";
            formLoading = false;
            return;
        }
        if (!formData.supported_languages.includes(formData.game_language)) {
            formError = "Game code language must be one of the supported languages";
            formLoading = false;
            return;
        }
        try {
            if (isEditing && editingGame) {
                const updatePayload: UpdateGameRequest = {
                    ...formData,
                };
                await gameService.update(editingGame.id, updatePayload);
            } else {
                const { is_active, ...createPayload } = formData;
                await gameService.create(createPayload);
            }
            await loadGames();
            closeForm();
        } catch (err) {
            formError =
                err instanceof Error ? err.message : "Failed to save game";
        } finally {
            formLoading = false;
        }
    }
    async function handleDelete(game: Game) {
        if (
            !confirm(
                `Are you sure you want to delete "${game.name}"? This action cannot be undone.`,
            )
        ) {
            return;
        }
        try {
            await gameService.delete(game.id);
            await loadGames();
        } catch (err) {
            error =
                err instanceof Error ? err.message : "Failed to delete game";
        }
    }
    function formatDate(dateStr: string): string {
        return new Date(dateStr).toLocaleDateString("en-US", {
            year: "numeric",
            month: "short",
            day: "numeric",
        });
    }
</script>

<div class="container page">
    <div class="page-header">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="page-title">Game Management</h1>
                <p class="text-gray-500">Create and manage games</p>
            </div>
            <div class="flex gap-2">
                <Button variant="primary" label="Create New Game" onclick={openCreateForm} />
                <LinkButton variant="secondary" href="/admin" label="Back to Dashboard" />
            </div>
        </div>
    </div>
    {#if error}
        <div
            class="border p-6 shadow-sm bg-hatch bg-red-100 border-l-4 border-red-600 mb-4"
        >
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch text-center">
            <p class="text-gray-500">Loading games...</p>
        </div>
    {:else if games.length === 0}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch text-center">
            <p class="text-gray-500">No games found. Create your first game!</p>
        </div>
    {:else}
        <div class="grid grid-2">
            {#each games as game}
                <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
                    <div class="flex justify-between items-start mb-2">
                        <h3 class="font-bold text-lg">{game.name}</h3>
                        <span
                            class="badge {game.is_active
                                ? 'badge-accepted'
                                : 'badge-cancelled'}"
                        >
                            {game.is_active ? "Active" : "Inactive"}
                        </span>
                    </div>
                    <p class="text-gray-700 mb-4">{game.description}</p>
                    <div class="mb-4">
                        <h4 class="font-semibold text-sm text-gray-700 mb-2">
                            Supported Languages:
                        </h4>
                        <div class="flex gap-2">
                            {#each game.supported_languages as lang}
                                <span class="badge badge-scheduled">{lang}</span
                                >
                            {/each}
                        </div>
                    </div>
                    <p class="text-sm text-gray-500 mb-4">
                        Created: {formatDate(game.created_at)}
                    </p>
                    <div class="flex gap-2">
                        <Button
                            variant="secondary"
                            label="Edit"
                            onclick={() => openEditForm(game)}
                        />
                        <Button
                            variant="danger"
                            label="Delete"
                            onclick={() => handleDelete(game)}
                        />
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<!-- Create/Edit Form Modal -->
{#if showForm}
    <div
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-1000 p-4"
        role="button"
        tabindex="0"
        aria-label="Close game form"
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
            class="border border-[--border-color] p-6 shadow-sm bg-hatch max-w-175 w-full max-h-[90vh] overflow-y-auto"
        >
            <h2 class="font-bold text-xl mb-4">
                {isEditing ? "Edit Game" : "Create New Game"}
            </h2>
            {#if formError}
                <div
                    class="bg-red-100 border-l-4 border-red-600 p-4 mb-4"
                >
                    <p class="text-red-600">{formError}</p>
                </div>
            {/if}
            <form onsubmit={handleSubmit}>
                <div class="mb-4">
                    <label for="name" class="block mb-2 font-medium text-gray-dark">Game Name</label>
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
                <fieldset
                    class="border-0 p-0 m-0"
                >
                    <legend
                        class="text-sm font-semibold mb-1"
                    >
                        Supported Languages
                    </legend>
                    <div class="flex gap-4" aria-label="Supported languages">
                        {#each ["rust", "go", "c"] as lang}
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    checked={formData.supported_languages.includes(
                                        lang as ProgrammingLanguage,
                                    )}
                                    onchange={() =>
                                        toggleLanguage(
                                            lang as ProgrammingLanguage,
                                        )}
                                    disabled={formLoading}
                                />
                                <span>{lang}</span>
                            </label>
                        {/each}
                    </div>
                </fieldset>
                <div class="mb-4">
                    <label for="game-language" class="block mb-2 font-medium text-gray-dark">Game Code Language</label>
                    <select
                        id="game-language"
                        class="input"
                        bind:value={formData.game_language}
                        disabled={formLoading || formData.supported_languages.length === 0}
                        required
                    >
                        <option value="" disabled>
                            Select language...
                        </option>
                        {#each formData.supported_languages as lang}
                            <option value={lang}>{lang}</option>
                        {/each}
                    </select>
                </div>
                <div class="mb-4">
                    <label for="game-code" class="block mb-2 font-medium text-gray-dark">Game Code</label>
                    <textarea
                        id="game-code"
                        class="textarea font-mono text-sm"
                        bind:value={formData.game_code}
                        disabled={formLoading}
                        rows="6"
                        required
                    ></textarea>
                </div>
                <div class="mb-4">
                    <label for="rounds-per-match" class="block mb-2 font-medium text-gray-dark">Rounds per Match</label>
                    <input
                        id="rounds-per-match"
                        type="number"
                        class="input"
                        min="1"
                        max="100"
                        bind:value={formData.rounds_per_match}
                        disabled={formLoading}
                        required
                    />
                </div>
                <div class="mb-4">
                    <label for="repetitions" class="block mb-2 font-medium text-gray-dark">Repetitions</label>
                    <input
                        id="repetitions"
                        type="number"
                        class="input"
                        min="1"
                        max="100"
                        bind:value={formData.repetitions}
                        disabled={formLoading}
                        required
                    />
                </div>
                <div class="mb-4">
                    <label for="timeout-seconds" class="block mb-2 font-medium text-gray-dark">Match Timeout (seconds)</label>
                    <input
                        id="timeout-seconds"
                        type="number"
                        class="input"
                        min="1"
                        max="3600"
                        bind:value={formData.timeout_ms}
                        disabled={formLoading}
                        required
                    />
                </div>
                <div class="mb-4">
                    <label for="cpu-limit" class="block mb-2 font-medium text-gray-dark">CPU Limit</label>
                    <input
                        id="cpu-limit"
                        type="text"
                        class="input"
                        bind:value={formData.cpu_limit}
                        disabled={formLoading}
                        required
                    />
                </div>
                <div class="mb-4">
                    <label for="memory-limit" class="block mb-2 font-medium text-gray-dark">Memory Limit</label>
                    <input
                        id="memory-limit"
                        type="text"
                        class="input"
                        bind:value={formData.memory_limit_mb}
                        disabled={formLoading}
                        required
                    />
                </div>
                <div class="mb-4">
                    <label
                        class="flex items-center gap-2 cursor-pointer"
                    >
                        <input
                            type="checkbox"
                            bind:checked={formData.is_active}
                            disabled={formLoading}
                        />
                        <span>Active</span>
                    </label>
                </div>
                <div class="flex gap-2">
                    <Button
                        type="submit"
                        variant="primary"
                        disabled={formLoading}
                        label={formLoading
                            ? "Saving..."
                            : isEditing
                              ? "Update Game"
                              : "Create Game"}
                    />
                    <Button
                        type="button"
                        variant="secondary"
                        disabled={formLoading}
                        label="Cancel"
                        onclick={closeForm}
                    />
                </div>
            </form>
        </div>
    </div>
{/if}
