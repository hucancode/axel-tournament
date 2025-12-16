<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { gameSetterService } from "$lib/services/game-setter";
  import type { Game, GameTemplate, ProgrammingLanguage } from "$lib/types";

  let { user, isAuthenticated } = $derived($authStore);
  let gameId = $derived($page.params.id);

  let game: Game | null = $state(null);
  let templates: GameTemplate[] = $state([]);
  let activeTab: "info" | "dockerfile" | "templates" = $state("info");
  let loading = $state(true);
  let error = $state("");
  let success = $state("");

  // Dockerfile tab
  let dockerfileContent = $state("");
  let uploadingDockerfile = $state(false);

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
      game = await api.get<Game>(`/api/admin/games/${gameId}`, true);
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

  async function uploadDockerfile() {
    if (!dockerfileContent.trim()) {
      error = "Dockerfile content cannot be empty";
      return;
    }

    try {
      uploadingDockerfile = true;
      error = "";
      success = "";

      await gameSetterService.uploadDockerfile(gameId, {
        dockerfile_content: dockerfileContent,
      });

      success = "Dockerfile uploaded successfully!";
      await loadGame();
    } catch (e: any) {
      error = e.message || "Failed to upload Dockerfile";
    } finally {
      uploadingDockerfile = false;
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
</script>

<div class="page">
  <div class="container">
    <button class="btn btn-secondary" on:click={() => goto("/game-setter")} style="margin-bottom: 1rem;">
      ← Back to Dashboard
    </button>

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
      <div style="display: flex; gap: 1rem; border-bottom: 2px solid #ddd; margin-bottom: 2rem;">
        <button
          class="tab-button {activeTab === 'info' ? 'active' : ''}"
          on:click={() => (activeTab = "info")}
        >
          Game Info
        </button>
        <button
          class="tab-button {activeTab === 'dockerfile' ? 'active' : ''}"
          on:click={() => (activeTab = "dockerfile")}
        >
          Dockerfile
        </button>
        <button
          class="tab-button {activeTab === 'templates' ? 'active' : ''}"
          on:click={() => (activeTab = "templates")}
        >
          Templates
        </button>
      </div>

      <!-- Tab Content -->
      {#if activeTab === "info"}
        <div class="card">
          <h2>Basic Information</h2>
          <dl style="display: grid; grid-template-columns: 150px 1fr; gap: 1rem;">
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

            <dt><strong>Dockerfile:</strong></dt>
            <dd>{game.dockerfile_path ? "✓ Uploaded" : "Not uploaded yet"}</dd>

            <dt><strong>Created:</strong></dt>
            <dd>{new Date(game.created_at).toLocaleDateString()}</dd>
          </dl>

          <div style="margin-top: 2rem;">
            <h3>Rules</h3>
            <pre style="background: #f5f5f5; padding: 1rem; border-radius: 4px; overflow-x: auto;">{JSON.stringify(
              game.rules,
              null,
              2
            )}</pre>
          </div>
        </div>
      {:else if activeTab === "dockerfile"}
        <div class="card">
          <h2>Upload Dockerfile</h2>
          <p class="text-sm">This Dockerfile will be used to build the execution environment for matches.</p>

          {#if game.dockerfile_path}
            <p class="text-sm">
              <strong>Current Dockerfile:</strong> {game.dockerfile_path}
            </p>
          {/if}

          <div class="form-group" style="margin-top: 1rem;">
            <label for="dockerfile">Dockerfile Content</label>
            <textarea
              id="dockerfile"
              class="textarea"
              bind:value={dockerfileContent}
              rows="15"
              placeholder="FROM rust:1.92-slim&#10;&#10;WORKDIR /workspace&#10;&#10;# Install dependencies...&#10;&#10;CMD [&quot;/runner.sh&quot;]"
              style="font-family: monospace; font-size: 0.9em;"
            />
          </div>

          <button class="btn btn-primary" on:click={uploadDockerfile} disabled={uploadingDockerfile}>
            {uploadingDockerfile ? "Uploading..." : "Upload Dockerfile"}
          </button>
        </div>
      {:else if activeTab === "templates"}
        <div class="card">
          <h2>Code Templates</h2>
          <p class="text-sm">Provide starter code templates for each supported language.</p>

          <div style="margin-top: 2rem; display: flex; flex-direction: column; gap: 2rem;">
            {#each game.supported_languages as lang}
              <div>
                <h3 style="text-transform: capitalize;">{lang} Template</h3>
                <textarea
                  class="textarea"
                  bind:value={templatesByLang[lang]}
                  rows="12"
                  placeholder={"fn main() {}"}
                  style="font-family: monospace; font-size: 0.9em;"
                />
                <button
                  class="btn btn-primary"
                  on:click={() => saveTemplate(lang)}
                  disabled={savingTemplate[lang]}
                  style="margin-top: 0.5rem;"
                >
                  {savingTemplate[lang] ? "Saving..." : `Save ${lang} Template`}
                </button>
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
