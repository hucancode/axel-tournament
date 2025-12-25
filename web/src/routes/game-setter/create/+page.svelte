<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { CreateGameRequest, GameType, ProgrammingLanguage } from '$lib/types';
  import { defaultInteractiveFrontend, defaultBackendCode } from '$lib/components/templates/game-templates';
  import LanguageSelector from '$lib/components/LanguageSelector.svelte';
  import Alert from '$lib/components/Alert.svelte';
  
  let gameType = $state<GameType>('automated');
  let name = $state('');
  let description = $state('');
  let supportedLanguages = $state<ProgrammingLanguage[]>(['rust']);
  let gameCode = $state('');
  let frontendCode = $state('');
  let gameLanguage = $state<ProgrammingLanguage>('rust');
  let roundsPerMatch = $state(10);
  let repetitions = $state(1);
  let timeoutMs = $state(1000);
  let cpuLimit = $state(0.5);
  let turnTimeoutMs = $state(100);
  let memoryLimitMb = $state(64);
  
  let creating = $state(false);
  let error = $state('');
  
  const languages: ProgrammingLanguage[] = ['rust', 'go', 'c'];
  
  onMount(() => {
    if (gameType === 'interactive') {
      frontendCode = defaultInteractiveFrontend;
      gameCode = defaultBackendCode;
    }
  });
  
  function onGameTypeChange() {
    if (gameType === 'interactive' && !frontendCode) {
      frontendCode = defaultInteractiveFrontend;
      gameCode = defaultBackendCode;
    }
  }
  
  function toggleLanguage(lang: ProgrammingLanguage) {
    if (supportedLanguages.includes(lang)) {
      supportedLanguages = supportedLanguages.filter(l => l !== lang);
    } else {
      supportedLanguages = [...supportedLanguages, lang];
    }
  }
  
  async function createGame(event: Event) {
    event.preventDefault();
    if (!name.trim() || !description.trim() || !gameCode.trim()) {
      error = 'Please fill in all required fields';
      return;
    }
    
    if (gameType === 'interactive' && !frontendCode.trim()) {
      error = 'Frontend code is required for interactive games';
      return;
    }
    
    if (supportedLanguages.length === 0) {
      error = 'Please select at least one supported language';
      return;
    }
    
    creating = true;
    error = '';
    
    try {
      const request: CreateGameRequest = {
        name: name.trim(),
        description: description.trim(),
        game_type: gameType,
        supported_languages: supportedLanguages,
        game_code: gameCode.trim(),
        game_language: gameLanguage,
        frontend_code: gameType === 'interactive' ? frontendCode.trim() : undefined,
        rounds_per_match: roundsPerMatch,
        repetitions: repetitions,
        timeout_ms: timeoutMs,
        cpu_limit: cpuLimit,
        turn_timeout_ms: turnTimeoutMs,
        memory_limit_mb: memoryLimitMb,
      };
      
      await api.post('/api/admin/games', request, true);
      goto('/game-setter');
    } catch (e: any) {
      error = e.message || 'Failed to create game';
    } finally {
      creating = false;
    }
  }
</script>

<div class="create-game">
  <div class="header">
    <h1>Create New Game</h1>
    <button class="btn-secondary" onclick={() => goto('/game-setter')}>
      Back to Dashboard
    </button>
  </div>
  
  {#if error}
    <Alert type="error" message={error} />
  {/if}
  
  <form onsubmit={createGame}>
    <fieldset>
      <legend>Basic Information</legend>
      
      <label for="game-type">Game Type</label>
      <select id="game-type" bind:value={gameType} onchange={onGameTypeChange}>
        <option value="automated">Automated (Code vs Code)</option>
        <option value="interactive">Interactive (Human vs Human)</option>
      </select>
      
      <label for="name">Game Name</label>
      <input id="name" type="text" bind:value={name} placeholder="Enter game name" required />
      
      <label for="description">Description</label>
      <textarea id="description" bind:value={description} placeholder="Describe your game" required></textarea>
      
      <LanguageSelector 
        {languages} 
        selected={supportedLanguages} 
        onToggle={toggleLanguage} 
      />
    </fieldset>
    
    <fieldset>
      <legend>Game Code</legend>
      
      <label for="game-language">Backend Language</label>
      <select id="game-language" bind:value={gameLanguage}>
        {#each languages as lang}
          <option value={lang}>{lang.toUpperCase()}</option>
        {/each}
      </select>
      
      <label for="game-code">Backend Code (Game Logic)</label>
      <textarea
        id="game-code"
        bind:value={gameCode}
        placeholder="Enter your game server code"
        rows="15"
        required
      ></textarea>
      
      {#if gameType === 'interactive'}
        <label for="frontend-code">Frontend Code (HTML/CSS/JS)</label>
        <textarea
          id="frontend-code"
          bind:value={frontendCode}
          placeholder="Enter your game frontend code"
          rows="20"
          required
        ></textarea>
      {/if}
    </fieldset>
    
    <fieldset>
      <legend>Game Settings</legend>
      
      <div class="row">
        <label for="rounds">Rounds per Match</label>
        <input id="rounds" type="number" bind:value={roundsPerMatch} min="1" max="100" />
        
        <label for="repetitions">Repetitions</label>
        <input id="repetitions" type="number" bind:value={repetitions} min="1" max="100" />
      </div>
      
      <div class="row">
        <label for="timeout">Timeout (ms)</label>
        <input id="timeout" type="number" bind:value={timeoutMs} min="100" max="5000" />
        
        <label for="cpu-limit">CPU Limit</label>
        <input id="cpu-limit" type="number" bind:value={cpuLimit} min="0.1" max="64" step="0.1" />
      </div>
      
      <div class="row">
        <label for="turn-timeout">Turn Timeout (ms)</label>
        <input id="turn-timeout" type="number" bind:value={turnTimeoutMs} min="1" max="2000" />
        
        <label for="memory-limit">Memory Limit (MB)</label>
        <input id="memory-limit" type="number" bind:value={memoryLimitMb} min="1" max="8192" />
      </div>
    </fieldset>
    
    <div class="form-actions">
      <button type="button" class="btn-secondary" onclick={() => goto('/game-setter')}>
        Cancel
      </button>
      <button type="submit" class="btn-primary" disabled={creating}>
        {creating ? 'Creating...' : 'Create Game'}
      </button>
    </div>
  </form>
</div>

<style>
  .create-game {
    max-width: 1000px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }
  
  fieldset {
    margin-bottom: 2rem;
    padding: 1.5rem;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
  }
  
  legend {
    padding: 0 0.5rem;
    font-weight: 600;
    color: #333;
  }
  
  label {
    display: block;
    margin: 1rem 0 0.5rem 0;
    font-weight: 500;
  }
  
  label:first-of-type {
    margin-top: 0;
  }
  
  input, select, textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: inherit;
  }
  
  textarea {
    font-family: 'Courier New', monospace;
    resize: vertical;
  }
  
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    align-items: end;
  }
  
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 2rem;
  }
  
  .btn-primary, .btn-secondary {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }
  
  .btn-primary {
    background: #1976d2;
    color: white;
  }
  
  .btn-primary:disabled {
    background: #ccc;
    cursor: not-allowed;
  }
  
  .btn-secondary {
    background: #f5f5f5;
    color: #333;
    border: 1px solid #ddd;
  }
</style>
