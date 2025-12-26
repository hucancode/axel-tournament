<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { gameSetterService } from "$lib/services/game-setter";
  import type { ProgrammingLanguage } from "$lib/types";

  let { user, isAuthenticated } = $derived($authStore);

  let name = $state("");
  let description = $state("");
  let selectedLanguages: ProgrammingLanguage[] = $state([]);
  let gameCodeContent = $state("");
  let gameLanguage: ProgrammingLanguage | "" = $state("");
  let roundsPerMatch = $state(3);
  let repetitions = $state(1);
  let timeoutMs = $state(2000);
  let cpuLimit = $state(1.0);
  let memoryLimit = $state(512);
  let turnTimeoutMs = $state(200);
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
      if (!gameCodeContent.trim() || !gameLanguage) {
        error = "Game code and language are required";
        loading = false;
        return;
      }


      const game = await gameSetterService.createGame({
        name,
        description,
        game_type: "automated",
        supported_languages: selectedLanguages,
        game_code: gameCodeContent,
        game_language: gameLanguage as ProgrammingLanguage,
        rounds_per_match: roundsPerMatch,
        repetitions,
        timeout_ms: timeoutMs,
        cpu_limit: cpuLimit,
        memory_limit_mb: memoryLimit,
        turn_timeout_ms: turnTimeoutMs,
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

      <fieldset class="form-group border-none p-0">
        <legend class="font-semibold mb-1">Supported Languages *</legend>
        <div class="flex gap-4 flex-wrap">
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              checked={selectedLanguages.includes("rust")}
              onchange={() => toggleLanguage("rust")}
            />
            Rust
          </label>
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              checked={selectedLanguages.includes("go")}
              onchange={() => toggleLanguage("go")}
            />
            Go
          </label>
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              checked={selectedLanguages.includes("c")}
              onchange={() => toggleLanguage("c")}
            />
            C
          </label>
        </div>
      </fieldset>

      <div class="form-group">
        <label for="game-lang">Game Code Language *</label>
        <select
          id="game-lang"
          class="input"
          bind:value={gameLanguage}
          required
        >
          <option value="">Select language...</option>
          <option value="rust">Rust</option>
          <option value="go">Go</option>
          <option value="c">C</option>
        </select>
      </div>

      <div class="form-group">
        <label for="game-code">Game Code *</label>
        <textarea
          id="game-code"
          class="textarea font-mono text-sm"
          bind:value={gameCodeContent}
          rows="10"
          placeholder="Your game orchestration code..."
          required
        ></textarea>
      </div>

      <div class="form-group">
        <label for="rounds">Rounds per Match *</label>
        <input
          id="rounds"
          type="number"
          class="input"
          min="1"
          max="100"
          bind:value={roundsPerMatch}
          required
        />
      </div>

      <div class="form-group">
        <label for="repetitions">Repetitions *</label>
        <input
          id="repetitions"
          type="number"
          class="input"
          min="1"
          max="100"
          bind:value={repetitions}
          required
        />
      </div>

      <div class="form-group">
        <label for="timeout-ms">Match Timeout (miliseconds) *</label>
        <input
          id="timeout-ms"
          type="number"
          class="input"
          min="1"
          max="3600"
          bind:value={timeoutMs}
          required
        />
      </div>

      <div class="form-group">
        <label for="cpu-limit">CPU Limit *</label>
        <input
          id="cpu-limit"
          type="text"
          class="input"
          bind:value={cpuLimit}
          required
        />
      </div>

      <div class="form-group">
        <label for="memory-limit">Memory Limit *</label>
        <input
          id="memory-limit"
          type="text"
          class="input"
          bind:value={memoryLimit}
          required
        />
      </div>

      <div class="flex gap-4">
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
