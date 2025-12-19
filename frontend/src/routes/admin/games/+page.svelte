<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { gameService } from "$lib/services/games";
    import type {
        Game,
        ProgrammingLanguage,
        CreateGameRequest,
    } from "$lib/types";
    let games = $state<Game[]>([]);
    let loading = $state(true);
    let error = $state("");
    // Create/Edit form state
    let showForm = $state(false);
    let editingGame = $state<Game | null>(null);
    let formData = $state<CreateGameRequest>({
        name: "",
        description: "",
        supported_languages: [],
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
            supported_languages: [],
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
            supported_languages: [...game.supported_languages],
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
        try {
            if (isEditing && editingGame) {
                await gameService.update(editingGame.id, formData);
            } else {
                await gameService.create(formData);
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
                <button class="btn btn-primary" onclick={openCreateForm}>
                    Create New Game
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
            <p class="text-gray-500">Loading games...</p>
        </div>
    {:else if games.length === 0}
        <div class="card text-center">
            <p class="text-gray-500">No games found. Create your first game!</p>
        </div>
    {:else}
        <div class="grid grid-2">
            {#each games as game}
                <div class="card">
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
                        <button
                            class="btn btn-secondary"
                            style="padding: 0.375rem 0.75rem; font-size: 0.875rem;"
                            onclick={() => openEditForm(game)}
                        >
                            Edit
                        </button>
                        <button
                            class="btn btn-danger"
                            style="padding: 0.375rem 0.75rem; font-size: 0.875rem;"
                            onclick={() => handleDelete(game)}
                        >
                            Delete
                        </button>
                    </div>
                </div>
            {/each}
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
            class="card"
            style="
				max-width: 700px;
				width: 100%;
				max-height: 90vh;
				overflow-y: auto;
			"
        >
            <h2 class="font-bold text-xl mb-4">
                {isEditing ? "Edit Game" : "Create New Game"}
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
                    <label for="name">Game Name</label>
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
                <fieldset
                    class="form-group"
                    style="border: none; padding: 0; margin: 0;"
                >
                    <legend
                        class="text-sm font-semibold"
                        style="margin-bottom: 0.25rem;"
                    >
                        Supported Languages
                    </legend>
                    <div class="flex gap-4" aria-label="Supported languages">
                        {#each ["rust", "go", "c"] as lang}
                            <label
                                class="flex items-center gap-2"
                                style="cursor: pointer;"
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
                <div class="form-group">
                    <label
                        class="flex items-center gap-2"
                        style="cursor: pointer;"
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
                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled={formLoading}
                    >
                        {formLoading
                            ? "Saving..."
                            : isEditing
                              ? "Update Game"
                              : "Create Game"}
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
