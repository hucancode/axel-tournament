<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { gameSetterService } from "$lib/services/game-setter";
  import type { Game, GameTemplate, ProgrammingLanguage } from "$lib/types";
  import { Button } from "$lib/components";

  let { user, isAuthenticated } = $derived($authStore);
  const gameId = $derived(page.params.id!);

  let game: Game | null = $state(null);
  let templates: GameTemplate[] = $state([]);
  let activeTab: "info" | "gamecode" | "templates" = $state("info");
  let loading = $state(true);
  let error = $state("");
  let success = $state("");

  // Edit mode for info tab
  let editMode = $state(false);
  let editForm = $state({
    name: "",
    description: "",
    supported_languages: [] as ProgrammingLanguage[],
    is_active: true,
    rounds_per_match: 3,
    repetitions: 1,
    timeout_ms: 120,
    cpu_limit: 1.0,
    memory_limit_mb: 2,
    turn_timeout_ms: 200,
  });

  // Game Code tab
  let gameCodeContent = $state("");
  let selectedGameLang: ProgrammingLanguage | "" = $state("");
  let uploadingGameCode = $state(false);

  // Templates tab
  let templatesByLang: Record<string, string> = $state({});
  let savingTemplate: Record<string, boolean> = $state({});

  // Redirect if not game setter or admin
  $effect(() => {
    if (!isAuthenticated || (user?.role !== "gamesetter" && user?.role !== "admin")) {
      goto("/");
    }
  });

  onMount(async () => {
    await loadGame();
    await loadTemplates();
  });

  async function loadGame() {
    try {
      loading = true;
      error = "";
      game = await api.get<Game>(`/api/games/${gameId}`);
      if (game) {
        if (!gameCodeContent && game.game_code) {
          gameCodeContent = game.game_code;
        }
        if (!selectedGameLang && game.game_language) {
          selectedGameLang = game.game_language;
        }
      }
    } catch (e: any) {
      error = e.message || "Failed to load game";
    } finally {
      loading = false;
    }
  }

  async function loadTemplates() {
    if (!game) return;

    try {
      templates = await gameSetterService.listTemplates(gameId);

      // Initialize template content for each supported language
      const tempByLang: Record<string, string> = {};
      for (const lang of game.supported_languages) {
        const existing = templates.find((t) => t.language === lang);
        tempByLang[lang] = existing?.template_code || "";
      }
      templatesByLang = tempByLang;
    } catch (e: any) {
      console.error("Failed to load templates:", e);
    }
  }

  async function saveTemplate(lang: ProgrammingLanguage) {
    const code = templatesByLang[lang];
    if (!code.trim()) {
      error = "Template code cannot be empty";
      return;
    }

    try {
      savingTemplate[lang] = true;
      error = "";
      success = "";

      const existing = templates.find((t) => t.language === lang);
      if (existing) {
        await gameSetterService.updateTemplate(gameId, lang, code);
      } else {
        await gameSetterService.createTemplate({
          game_id: gameId,
          language: lang,
          template_code: code,
        });
      }

      success = `Template for ${lang} saved successfully!`;
      await loadTemplates();
    } catch (e: any) {
      error = e.message || `Failed to save template for ${lang}`;
    } finally {
      savingTemplate[lang] = false;
    }
  }

  function enableEditMode() {
    if (!game) return;
    editForm = {
      name: game.name,
      description: game.description,
      supported_languages: [...game.supported_languages],
      is_active: game.is_active,
      rounds_per_match: game.rounds_per_match,
      repetitions: game.repetitions,
      timeout_ms: game.timeout_ms,
      cpu_limit: game.cpu_limit,
      memory_limit_mb: game.memory_limit_mb,
      turn_timeout_ms: game.turn_timeout_ms,
    };
    editMode = true;
  }

  function cancelEdit() {
    editMode = false;
  }

  async function saveGameEdits() {
    try {
      error = "";
      success = "";
      await gameSetterService.updateGame(gameId, editForm);
      success = "Game updated successfully!";
      editMode = false;
      await loadGame();
    } catch (e: any) {
      error = e.message || "Failed to update game";
    }
  }

  async function deleteGame() {
    if (!confirm("Are you sure you want to delete this game? This action cannot be undone.")) {
      return;
    }

    try {
      error = "";
      await gameSetterService.deleteGame(gameId);
      goto("/game-setter");
    } catch (e: any) {
      error = e.message || "Failed to delete game";
    }
  }

  async function uploadGameCode() {
    if (!gameCodeContent.trim()) {
      error = "Game code cannot be empty";
      return;
    }

    if (!selectedGameLang) {
      error = "Please select a language";
      return;
    }

    try {
      uploadingGameCode = true;
      error = "";
      success = "";

      await gameSetterService.updateGame(gameId, {
        game_code: gameCodeContent,
        game_language: selectedGameLang as ProgrammingLanguage
      });

      success = "Game code updated successfully!";
      await loadGame();
    } catch (e: any) {
      error = e.message || "Failed to update game code";
    } finally {
      uploadingGameCode = false;
    }
  }
</script>

<div class="page">
  <div class="container">
    <Button variant="secondary" label="← Back to Dashboard" onclick={() => goto("/game-setter")} />
    <div class="mb-4"></div>

    <h1>{game?.name || "Loading..."}</h1>

    {#if error}
      <p class="error-message">{error}</p>
    {/if}

    {#if success}
      <p class="success-message">{success}</p>
    {/if}

    {#if loading}
      <p>Loading game...</p>
    {:else if game}
      <!-- Tabs -->
      <div class="flex gap-4 border-b-2 border-gray-300 mb-8">
        <button
          type="button"
          class="tab-button {activeTab === 'info' ? 'active' : ''}"
          onclick={() => (activeTab = "info")}
        >
          Game Info
        </button>
        <button
          type="button"
          class="tab-button {activeTab === 'gamecode' ? 'active' : ''}"
          onclick={() => (activeTab = "gamecode")}
        >
          Game Code
        </button>
        <button
          type="button"
          class="tab-button {activeTab === 'templates' ? 'active' : ''}"
          onclick={() => (activeTab = "templates")}
        >
          Templates
        </button>
      </div>

      <!-- Tab Content -->
      {#if activeTab === "info"}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
          <div class="flex justify-between items-center mb-6">
            <h2>Basic Information</h2>
            <div class="flex gap-2">
              {#if !editMode}
                <Button variant="secondary" label="Edit" onclick={enableEditMode} />
                <Button variant="danger" label="Delete" onclick={deleteGame} />
              {/if}
            </div>
          </div>

          {#if editMode}
            <!-- Edit Mode -->
            <div class="mb-4">
              <label for="edit-name" class="block mb-2 font-medium text-gray-dark">Name</label>
              <input type="text" id="edit-name" class="input" bind:value={editForm.name} />
            </div>

            <div class="mb-4">
              <label for="edit-description" class="block mb-2 font-medium text-gray-dark">Description</label>
              <textarea
                id="edit-description"
                class="textarea"
                bind:value={editForm.description}
                rows="3"
              ></textarea>
            </div>

            <div class="mb-4">
              <label for="edit-rounds" class="block mb-2 font-medium text-gray-dark">Rounds per Match</label>
              <input
                type="number"
                id="edit-rounds"
                class="input"
                min="1"
                max="100"
                bind:value={editForm.rounds_per_match}
              />
            </div>

            <div class="mb-4">
              <label for="edit-repetitions" class="block mb-2 font-medium text-gray-dark">Repetitions</label>
              <input
                type="number"
                id="edit-repetitions"
                class="input"
                min="1"
                max="100"
                bind:value={editForm.repetitions}
              />
            </div>

            <div class="mb-4">
              <label for="edit-timeout" class="block mb-2 font-medium text-gray-dark">Match Timeout (seconds)</label>
              <input
                type="number"
                id="edit-timeout"
                class="input"
                min="1"
                max="3600"
                bind:value={editForm.timeout_ms}
              />
            </div>

            <div class="mb-4">
              <label for="edit-cpu-limit" class="block mb-2 font-medium text-gray-dark">CPU Limit</label>
              <input
                type="text"
                id="edit-cpu-limit"
                class="input"
                bind:value={editForm.cpu_limit}
              />
            </div>

            <div class="mb-4">
              <label for="edit-memory-limit" class="block mb-2 font-medium text-gray-dark">Memory Limit</label>
              <input
                type="text"
                id="edit-memory-limit"
                class="input"
                bind:value={editForm.memory_limit_mb}
              />
            </div>

            <div class="mb-4">
              <label>
                <input type="checkbox" bind:checked={editForm.is_active} />
                Active
              </label>
            </div>

            <div class="flex gap-2 mt-4">
              <Button variant="primary" label="Save Changes" onclick={saveGameEdits} />
              <Button variant="secondary" label="Cancel" onclick={cancelEdit} />
            </div>
          {:else}
            <!-- View Mode -->
            <dl class="grid grid-cols-[150px_1fr] gap-4">
              <dt><strong>Description:</strong></dt>
              <dd>{game.description}</dd>

              <dt><strong>Status:</strong></dt>
              <dd>
                <span class="badge {game.is_active ? 'badge-running' : 'badge-cancelled'}">
                  {game.is_active ? "Active" : "Inactive"}
                </span>
              </dd>

              <dt><strong>Languages:</strong></dt>
              <dd>{game.supported_languages.join(", ")}</dd>

              <dt><strong>Rounds per Match:</strong></dt>
              <dd>{game.rounds_per_match}</dd>

              <dt><strong>Repetitions:</strong></dt>
              <dd>{game.repetitions}</dd>

              <dt><strong>Match Timeout:</strong></dt>
              <dd>{game.timeout_ms}s</dd>

              <dt><strong>CPU Limit:</strong></dt>
              <dd>{game.cpu_limit}</dd>

              <dt><strong>Memory Limit:</strong></dt>
              <dd>{game.memory_limit_mb}</dd>

              <dt><strong>Game Code:</strong></dt>
              <dd>{game.game_code ? `✓ Uploaded (${game.game_language})` : "Not uploaded yet"}</dd>

              <dt><strong>Created:</strong></dt>
              <dd>{new Date(game.created_at).toLocaleDateString()}</dd>
            </dl>
          {/if}
        </div>
      {:else if activeTab === "gamecode"}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
          <h2>Upload Game Code</h2>
          <div class="text-sm">
            This is your main game orchestration code that will:
            <ul class="my-2 pl-6">
              <li>Invoke player binaries via stdin/stdout</li>
              <li>Implement game logic and rules</li>
              <li>Handle player timeouts/crashes/invalid responses</li>
              <li>Output score array to stdout</li>
            </ul>
          </div>

          <p class="text-sm">
            <strong>Output Protocol:</strong> Print scores/error codes separated by spaces or newlines.
            Example: <code>100 TLE 92 WA</code> or one per line.
            Error codes: TLE (timeout), WA (wrong answer), CE (compiler error), RE (runtime error).
          </p>

          {#if game.game_code}
            <p class="text-sm">
              <strong>Current Game Code:</strong> Uploaded ({game.game_language})
            </p>
          {/if}

          <div class="mt-4 mb-4">
            <label for="game-lang" class="block mb-2 font-medium text-gray-dark">Programming Language</label>
            <select id="game-lang" class="input" bind:value={selectedGameLang}>
              <option value="">Select language...</option>
              {#each game.supported_languages as lang}
                <option value={lang}>{lang}</option>
              {/each}
            </select>
          </div>

          <div class="mb-4">
            <label for="game-code" class="block mb-2 font-medium text-gray-dark">Game Code</label>
            <textarea
              id="game-code"
              class="textarea font-mono text-sm"
              bind:value={gameCodeContent}
              rows="20"
              placeholder="Your game orchestration code..."
            ></textarea>
          </div>

          <Button variant="primary" label={uploadingGameCode ? "Uploading..." : "Upload Game Code"} onclick={uploadGameCode} disabled={uploadingGameCode} />
        </div>
      {:else if activeTab === "templates"}
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
          <h2>Code Templates</h2>
          <p class="text-sm">Provide starter code templates for each supported language.</p>

          <div class="mt-8 flex flex-col gap-8">
            {#each game.supported_languages as lang}
              <div>
                <h3 class="capitalize">{lang} Template</h3>
                <textarea
                  class="textarea font-mono text-sm"
                  bind:value={templatesByLang[lang]}
                  rows="12"
                  placeholder={"fn main() {}"}
                ></textarea>
                <Button
                  variant="primary"
                  label={savingTemplate[lang] ? "Saving..." : `Save ${lang} Template`}
                  onclick={() => saveTemplate(lang)}
                  disabled={savingTemplate[lang]}
                />
              </div>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .tab-button {
    padding: 0.75rem 1.5rem;
    background: none;
    border: none;
    border-bottom: 3px solid transparent;
    cursor: pointer;
    font-size: 1rem;
    color: #666;
    transition: all 0.2s;
  }

  .tab-button:hover {
    color: #333;
  }

  .tab-button.active {
    color: #007bff;
    border-bottom-color: #007bff;
  }

  dl {
    margin: 0;
  }

  dt {
    font-weight: bold;
  }

  dd {
    margin: 0;
  }

  h3 {
    margin: 0 0 0.5rem 0;
  }
</style>
