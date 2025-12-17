<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { gameSetterService } from "$lib/services/game-setter";
  import type { ProgrammingLanguage } from "$lib/types";

  let { user, isAuthenticated } = $derived($authStore);

  let name = $state("");
  let description = $state("");
  let rulesJson = $state("{}");
  let selectedLanguages: ProgrammingLanguage[] = $state([]);
  let error = $state("");
  let loading = $state(false);

  // Redirect if not game setter or admin
  $effect(() => {
    if (!isAuthenticated || (user?.role !== "gamesetter" && user?.role !== "admin")) {
      goto("/");
    }
  });

  function toggleLanguage(lang: ProgrammingLanguage) {
    if (selectedLanguages.includes(lang)) {
      selectedLanguages = selectedLanguages.filter((l) => l !== lang);
    } else {
      selectedLanguages = [...selectedLanguages, lang];
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";

    try {
      let rules: Record<string, any>;
      try {
        rules = JSON.parse(rulesJson);
      } catch {
        throw new Error("Invalid JSON in rules field");
      }

      const game = await gameSetterService.createGame({
        name,
        description,
        rules,
        supported_languages: selectedLanguages,
        is_active: true,
      });

      goto(`/game-setter/games/${game.id}`);
    } catch (e: any) {
      error = e.message || "Failed to create game";
    } finally {
      loading = false;
    }
  }
</script>

<div class="page">
  <div class="container">
    <h1>Create New Game</h1>

    {#if error}
      <p class="error-message">{error}</p>
    {/if}

    <form onsubmit={handleSubmit}>
      <div class="form-group">
        <label for="name">Game Name *</label>
        <input
          id="name"
          type="text"
          class="input"
          bind:value={name}
          required
          placeholder="e.g., Chess AI Battle"
        />
      </div>

      <div class="form-group">
        <label for="description">Description *</label>
        <textarea
          id="description"
          class="textarea"
          bind:value={description}
          required
          rows="3"
          placeholder="Describe your game..."
        ></textarea>
      </div>

      <div class="form-group">
        <label for="rules">Rules (JSON) *</label>
        <textarea
          id="rules"
          class="textarea"
          bind:value={rulesJson}
          required
          rows="5"
          placeholder={'{"rule1": "value1", "rule2": "value2"}'}
        ></textarea>
      </div>

      <fieldset class="form-group" style="border: none; padding: 0;">
        <legend style="font-weight: 600; margin-bottom: 0.25rem;">Supported Languages *</legend>
        <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
          <label style="display: flex; align-items: center; gap: 0.5rem;">
            <input
              type="checkbox"
              checked={selectedLanguages.includes("rust")}
              onchange={() => toggleLanguage("rust")}
            />
            Rust
          </label>
          <label style="display: flex; align-items: center; gap: 0.5rem;">
            <input
              type="checkbox"
              checked={selectedLanguages.includes("go")}
              onchange={() => toggleLanguage("go")}
            />
            Go
          </label>
          <label style="display: flex; align-items: center; gap: 0.5rem;">
            <input
              type="checkbox"
              checked={selectedLanguages.includes("c")}
              onchange={() => toggleLanguage("c")}
            />
            C
          </label>
        </div>
      </fieldset>

      <div style="display: flex; gap: 1rem;">
        <button type="submit" class="btn btn-primary" disabled={loading || selectedLanguages.length === 0}>
          {loading ? "Creating..." : "Create Game"}
        </button>
        <button type="button" class="btn btn-secondary" onclick={() => goto("/game-setter")}>
          Cancel
        </button>
      </div>
    </form>
  </div>
</div>
