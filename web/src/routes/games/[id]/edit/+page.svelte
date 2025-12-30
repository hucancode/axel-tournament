<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { gameService } from "$lib/services/games";
    import { Button, LinkButton, LoadingCard } from "$lib/components";
    import type { Game, ProgrammingLanguage, UpdateGameRequest } from "$lib/types";

    let gameId = $derived($page.params.id);
    let game = $state<Game | null>(null);
    let loading = $state(true);
    let error = $state("");

    type GameFormData = UpdateGameRequest & { is_active: boolean };
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
        memory_limit_mb: 512,
        turn_timeout_ms: 200,
        is_active: true,
    });
    let formLoading = $state(false);
    let formError = $state("");

    const auth = $derived($authStore);

    onMount(async () => {
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        await loadGame();
    });

    async function loadGame() {
        loading = true;
        error = "";
        try {
            game = await gameService.get(gameId);

            // Check permissions
            if (auth.user?.role !== "admin" && game.owner_id !== auth.user?.id) {
                goto("/games");
                return;
            }

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
        } catch (err) {
            error = err instanceof Error ? err.message : "Failed to load game";
        } finally {
            loading = false;
        }
    }

    function toggleLanguage(lang: ProgrammingLanguage) {
        if (formData.supported_languages!.includes(lang)) {
            formData.supported_languages = formData.supported_languages!.filter(
                (l) => l !== lang,
            );
        } else {
            formData.supported_languages = [
                ...(formData.supported_languages || []),
                lang,
            ];
        }
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        formLoading = true;
        formError = "";

        if (!formData.supported_languages || formData.supported_languages.length === 0) {
            formError = "Please select at least one supported language";
            formLoading = false;
            return;
        }

        if (!formData.game_code?.trim()) {
            formError = "Game code is required";
            formLoading = false;
            return;
        }

        if (formData.game_language && !formData.supported_languages.includes(formData.game_language)) {
            formError = "Game code language must be one of the supported languages";
            formLoading = false;
            return;
        }

        try {
            await gameService.update(gameId, formData);
            goto("/games");
        } catch (err) {
            formError = err instanceof Error ? err.message : "Failed to update game";
        } finally {
            formLoading = false;
        }
    }
</script>

<div class="container page">
    <div class="page-header">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="page-title">Edit Game</h1>
                <p class="text-gray-500">Update game configuration</p>
            </div>
            <LinkButton variant="secondary" href="/games" label="Back to Games" />
        </div>
    </div>

    {#if loading}
        <LoadingCard message="Loading game..." />
    {:else if error}
        <div class="bg-red-100 border-l-4 border-red-600 p-4 mb-4">
            <p class="text-red-600">{error}</p>
        </div>
    {:else if game}
        {#if formError}
            <div class="bg-red-100 border-l-4 border-red-600 p-4 mb-4">
                <p class="text-red-600">{formError}</p>
            </div>
        {/if}

        <form onsubmit={handleSubmit} class="border border-[--border-color] p-6 shadow-sm bg-hatch">
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

            <fieldset class="border-0 p-0 m-0 mb-4">
                <legend class="text-sm font-semibold mb-1">Supported Languages</legend>
                <div class="flex gap-4" aria-label="Supported languages">
                    {#each ["rust", "go", "c"] as lang}
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={formData.supported_languages?.includes(lang as ProgrammingLanguage)}
                                onchange={() => toggleLanguage(lang as ProgrammingLanguage)}
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
                    disabled={formLoading || !formData.supported_languages || formData.supported_languages.length === 0}
                    required
                >
                    <option value="" disabled>Select language...</option>
                    {#if formData.supported_languages}
                        {#each formData.supported_languages as lang}
                            <option value={lang}>{lang}</option>
                        {/each}
                    {/if}
                </select>
            </div>

            <div class="mb-4">
                <label for="game-code" class="block mb-2 font-medium text-gray-dark">Game Code</label>
                <textarea
                    id="game-code"
                    class="textarea font-mono text-sm"
                    bind:value={formData.game_code}
                    disabled={formLoading}
                    rows="10"
                    required
                ></textarea>
            </div>

            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
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

                <div>
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
            </div>

            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
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

                <div>
                    <label for="turn-timeout" class="block mb-2 font-medium text-gray-dark">Turn Timeout (ms)</label>
                    <input
                        id="turn-timeout"
                        type="number"
                        class="input"
                        min="1"
                        max="10000"
                        bind:value={formData.turn_timeout_ms}
                        disabled={formLoading}
                        required
                    />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
                    <label for="cpu-limit" class="block mb-2 font-medium text-gray-dark">CPU Limit</label>
                    <input
                        id="cpu-limit"
                        type="number"
                        step="0.1"
                        class="input"
                        bind:value={formData.cpu_limit}
                        disabled={formLoading}
                        required
                    />
                </div>

                <div>
                    <label for="memory-limit" class="block mb-2 font-medium text-gray-dark">Memory Limit (MB)</label>
                    <input
                        id="memory-limit"
                        type="number"
                        class="input"
                        bind:value={formData.memory_limit_mb}
                        disabled={formLoading}
                        required
                    />
                </div>
            </div>

            <div class="mb-4">
                <label class="flex items-center gap-2 cursor-pointer">
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
                    label={formLoading ? "Updating..." : "Update Game"}
                />
                <LinkButton
                    href="/games"
                    variant="secondary"
                    label="Cancel"
                />
            </div>
        </form>
    {/if}
</div>
