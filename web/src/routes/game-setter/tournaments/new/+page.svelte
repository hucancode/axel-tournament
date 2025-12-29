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
  import { Button, DateTimePicker, Select } from "$lib/components";

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
      <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
        <div class="mb-4">
          <Select
            label="Game *"
            options={[
              { value: "", label: "Select a game..." },
              ...myGames.map(game => ({ value: game.id, label: game.name }))
            ]}
            bind:value={form.game_id}
          />
          {#if myGames.length === 0}
            <p class="text-sm">You don't have any games yet. <a href="/game-setter/games/new">Create a game</a> first.</p>
          {/if}
        </div>

        <div class="mb-4">
          <label for="name" class="block mb-2 font-medium text-gray-dark">Tournament Name *</label>
          <input type="text" id="name" class="input" bind:value={form.name} placeholder="My Awesome Tournament" required />
        </div>

        <div class="mb-4">
          <label for="description" class="block mb-2 font-medium text-gray-dark">Description *</label>
          <textarea
            id="description"
            class="textarea"
            bind:value={form.description}
            rows="4"
            placeholder="Describe your tournament..."
            required
          ></textarea>
        </div>

        <div class="mb-4">
          <Select
            label="Match Generation Type *"
            options={[
              { value: "all_vs_all", label: "All vs All" },
              { value: "round_robin", label: "Round Robin" },
              { value: "single_elimination", label: "Single Elimination" },
              { value: "double_elimination", label: "Double Elimination" }
            ]}
            bind:value={form.match_generation_type}
          />
        </div>

        <div class="grid grid-2">
          <div class="mb-4">
            <label for="min-players" class="block mb-2 font-medium text-gray-dark">Minimum Players *</label>
            <input type="number" id="min-players" class="input" bind:value={form.min_players} min="2" required />
          </div>

          <div class="mb-4">
            <label for="max-players" class="block mb-2 font-medium text-gray-dark">Maximum Players *</label>
            <input type="number" id="max-players" class="input" bind:value={form.max_players} min="2" required />
          </div>
        </div>

        <div class="mb-4">
          <Select
            label="Initial Status *"
            options={[
              { value: "scheduled", label: "Scheduled" },
              { value: "registration", label: "Registration Open" }
            ]}
            bind:value={form.status}
          />
        </div>

        <div class="grid grid-2">
          <DateTimePicker
            label="Start Time (optional)"
            bind:value={form.start_time}
          />

          <DateTimePicker
            label="End Time (optional)"
            bind:value={form.end_time}
          />
        </div>

        <div class="flex gap-2 mt-6">
          <Button variant="primary" label={creating ? "Creating..." : "Create Tournament"} onclick={createTournament} disabled={creating} />
          <Button variant="secondary" label="Cancel" onclick={() => goto("/game-setter")} />
        </div>
      </div>
    {/if}
  </div>
</div>
