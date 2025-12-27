<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { gameSetterService } from "$lib/services/game-setter";
  import type {
    Game,
    CreateTournamentRequest,
    UpdateTournamentRequest,
    TournamentStatus,
    MatchGenerationType,
  } from "$lib/types";
  import { Button } from "$lib/components";

  let { user, isAuthenticated } = $derived($authStore);
  let myGames: Game[] = $state([]);
  let loading = $state(true);
  let creating = $state(false);
  let error = $state("");

  // Form data
  type TournamentFormData = CreateTournamentRequest & {
    status: TournamentStatus;
    match_generation_type: MatchGenerationType;
  };
  let form = $state<TournamentFormData>({
    game_id: "",
    name: "",
    description: "",
    status: "scheduled",
    min_players: 2,
    max_players: 10,
    start_time: "",
    end_time: "",
    match_generation_type: "all_vs_all",
  });

  // Redirect if not game setter or admin
  $effect(() => {
    if (!isAuthenticated || (user?.role !== "gamesetter" && user?.role !== "admin")) {
      goto("/");
    }
  });

  onMount(async () => {
    await loadMyGames();
  });

  async function loadMyGames() {
    try {
      loading = true;
      error = "";
      myGames = await gameSetterService.listMyGames();
    } catch (e: any) {
      error = e.message || "Failed to load games";
    } finally {
      loading = false;
    }
  }

  async function createTournament() {
    if (!form.game_id) {
      error = "Please select a game";
      return;
    }

    if (!form.name.trim()) {
      error = "Please enter a tournament name";
      return;
    }

    if (form.min_players < 2) {
      error = "Minimum players must be at least 2";
      return;
    }

    if (form.max_players < form.min_players) {
      error = "Maximum players must be greater than or equal to minimum players";
      return;
    }

    try {
      creating = true;
      error = "";

      const createPayload: CreateTournamentRequest = {
        game_id: form.game_id,
        name: form.name,
        description: form.description,
        min_players: form.min_players,
        max_players: form.max_players,
        start_time: form.start_time ? new Date(form.start_time).toISOString() : undefined,
        end_time: form.end_time ? new Date(form.end_time).toISOString() : undefined,
        match_generation_type: form.match_generation_type,
      };
      const tournament = await gameSetterService.createTournament(createPayload);

      if (form.status !== "scheduled") {
        const updatePayload: UpdateTournamentRequest = {
          status: form.status,
        };
        await gameSetterService.updateTournament(tournament.id, updatePayload);
      }

      goto(`/tournaments/${tournament.id}`);
    } catch (e: any) {
      error = e.message || "Failed to create tournament";
    } finally {
      creating = false;
    }
  }
</script>

<div class="page">
  <div class="container">
    <Button variant="secondary" label="← Back to Dashboard" onclick={() => goto("/game-setter")} />
    <div class="mb-4"></div>

    <h1>Create Tournament</h1>

    {#if error}
      <p class="error-message">{error}</p>
    {/if}

    {#if loading}
      <p>Loading...</p>
    {:else}
      <div class="card">
        <div class="form-group">
          <label for="game">Game *</label>
          <select id="game" class="input" bind:value={form.game_id} required>
            <option value="">Select a game...</option>
            {#each myGames as game}
              <option value={game.id}>{game.name}</option>
            {/each}
          </select>
          {#if myGames.length === 0}
            <p class="text-sm">You don't have any games yet. <a href="/game-setter/games/new">Create a game</a> first.</p>
          {/if}
        </div>

        <div class="form-group">
          <label for="name">Tournament Name *</label>
          <input type="text" id="name" class="input" bind:value={form.name} placeholder="My Awesome Tournament" required />
        </div>

        <div class="form-group">
          <label for="description">Description *</label>
          <textarea
            id="description"
            class="textarea"
            bind:value={form.description}
            rows="4"
            placeholder="Describe your tournament..."
            required
          ></textarea>
        </div>

        <div class="form-group">
          <label for="match-generation">Match Generation Type *</label>
          <select
            id="match-generation"
            class="input"
            bind:value={form.match_generation_type}
            required
          >
            <option value="all_vs_all">All vs All</option>
            <option value="round_robin">Round Robin</option>
            <option value="single_elimination">Single Elimination</option>
            <option value="double_elimination">Double Elimination</option>
          </select>
        </div>

        <div class="grid grid-2">
          <div class="form-group">
            <label for="min-players">Minimum Players *</label>
            <input type="number" id="min-players" class="input" bind:value={form.min_players} min="2" required />
          </div>

          <div class="form-group">
            <label for="max-players">Maximum Players *</label>
            <input type="number" id="max-players" class="input" bind:value={form.max_players} min="2" required />
          </div>
        </div>

        <div class="form-group">
          <label for="status">Initial Status *</label>
          <select id="status" class="input" bind:value={form.status}>
            <option value="scheduled">Scheduled</option>
            <option value="registration">Registration Open</option>
          </select>
        </div>

        <div class="grid grid-2">
          <div class="form-group">
            <label for="start-time">Start Time (optional)</label>
            <input type="datetime-local" id="start-time" class="input" bind:value={form.start_time} />
          </div>

          <div class="form-group">
            <label for="end-time">End Time (optional)</label>
            <input type="datetime-local" id="end-time" class="input" bind:value={form.end_time} />
          </div>
        </div>

        <div class="flex gap-2 mt-6">
          <Button variant="primary" label={creating ? "Creating..." : "Create Tournament"} onclick={createTournament} disabled={creating} />
          <Button variant="secondary" label="Cancel" onclick={() => goto("/game-setter")} />
        </div>
      </div>
    {/if}
  </div>
</div>
