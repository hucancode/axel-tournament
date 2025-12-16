<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { Game, Tournament } from "$lib/types";

  let { user, isAuthenticated } = $derived($authStore);
  let myGames: Game[] = $state([]);
  let myTournaments: Tournament[] = $state([]);
  let loading = $state(true);
  let error = $state("");

  // Redirect if not game setter or admin
  $effect(() => {
    if (!isAuthenticated || (user?.role !== "gamesetter" && user?.role !== "admin")) {
      goto("/");
    }
  });

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = "";

      // Load games owned by current user
      const allGames = await api.get<Game[]>("/api/admin/games", true);
      myGames = allGames.filter((game) => game.owner_id === user?.id || user?.role === "admin");

      // Load tournaments
      myTournaments = await api.get<Tournament[]>("/api/tournaments", true);
    } catch (e: any) {
      error = e.message || "Failed to load data";
    } finally {
      loading = false;
    }
  }

  function createGame() {
    goto("/game-setter/games/new");
  }

  function manageGame(gameId: string) {
    goto(`/game-setter/games/${gameId}`);
  }

  function createTournament() {
    goto("/game-setter/tournaments/new");
  }
</script>

<div class="page">
  <div class="container">
    <h1>Game Setter Dashboard</h1>

    {#if error}
      <p class="error-message">{error}</p>
    {/if}

    {#if loading}
      <p>Loading...</p>
    {:else}
      <div class="grid grid-2">
        <!-- My Games -->
        <div class="card">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
            <h2>My Games</h2>
            <button class="btn btn-primary" on:click={createGame}>Create Game</button>
          </div>

          {#if myGames.length === 0}
            <p class="text-sm">No games yet. Create your first game to get started!</p>
          {:else}
            <div style="display: flex; flex-direction: column; gap: 1rem;">
              {#each myGames as game}
                <div class="card" style="cursor: pointer;" on:click={() => manageGame(game.id)}>
                  <h3>{game.name}</h3>
                  <p class="text-sm">{game.description}</p>
                  <div style="margin-top: 0.5rem;">
                    <span class="badge {game.is_active ? 'badge-running' : 'badge-cancelled'}">
                      {game.is_active ? 'Active' : 'Inactive'}
                    </span>
                    {#if game.dockerfile_path}
                      <span class="badge badge-completed">Has Dockerfile</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- My Tournaments -->
        <div class="card">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
            <h2>My Tournaments</h2>
            <button class="btn btn-primary" on:click={createTournament}>Create Tournament</button>
          </div>

          {#if myTournaments.length === 0}
            <p class="text-sm">No tournaments yet.</p>
          {:else}
            <div style="display: flex; flex-direction: column; gap: 1rem;">
              {#each myTournaments as tournament}
                <div class="card" style="cursor: pointer;" on:click={() => goto(`/tournaments/${tournament.id}`)}>
                  <h3>{tournament.name}</h3>
                  <p class="text-sm">{tournament.description}</p>
                  <div style="margin-top: 0.5rem;">
                    <span class="badge badge-{tournament.status}">{tournament.status}</span>
                    <span class="text-xs">{tournament.current_players}/{tournament.max_players} players</span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  h3 {
    margin: 0 0 0.5rem 0;
  }
</style>
