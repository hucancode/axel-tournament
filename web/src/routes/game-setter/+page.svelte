<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { gameSetterService } from "$lib/services/game-setter";
  import type { Game, Tournament, TournamentStatus } from "$lib/types";

  let { user, isAuthenticated } = $derived($authStore);
  let myGames: Game[] = $state([]);
  let allTournaments: Tournament[] = $state([]);
  let loading = $state(true);
  let error = $state("");
  let actionError = $state("");
  let actionMessage = $state("");
  let tournamentAction = $state<Record<string, boolean>>({});

  // Computed: filter tournaments to only show those for user's games
  let myGameIds = $derived(myGames.map(g => g.id));
  let filteredTournaments = $derived(allTournaments.filter(t => myGameIds.includes(t.game_id)));

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

      // Load games owned by current user using game-setter API
      myGames = await gameSetterService.listMyGames();

      // Load all tournaments, will be filtered by myGameIds
      allTournaments = await api.get<Tournament[]>("/api/tournaments", true);
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

  function handleCardKeydown(event: KeyboardEvent, action: () => void) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      action();
    }
  }

  function getStatusBadge(status: TournamentStatus) {
    return `badge badge-${status}`;
  }

  function canOpenRegistration(status: TournamentStatus) {
    return status === "scheduled" || status === "cancelled";
  }

  function canStart(status: TournamentStatus) {
    return status === "registration";
  }

  function canClose(status: TournamentStatus) {
    return status === "running" || status === "registration";
  }

  function canCancel(status: TournamentStatus) {
    return status === "running" || status === "registration";
  }

  async function updateTournamentStatus(tournamentId: string, status: TournamentStatus) {
    tournamentAction = { ...tournamentAction, [tournamentId]: true };
    actionError = "";
    actionMessage = "";
    try {
      await gameSetterService.updateTournament(tournamentId, { status });
      actionMessage = `Tournament marked as ${status}.`;
      await loadData();
    } catch (e: any) {
      actionError = e.message || "Failed to update tournament status";
    } finally {
      tournamentAction = { ...tournamentAction, [tournamentId]: false };
    }
  }

  async function startTournament(tournamentId: string) {
    tournamentAction = { ...tournamentAction, [tournamentId]: true };
    actionError = "";
    actionMessage = "";
    try {
      await gameSetterService.startTournament(tournamentId);
      actionMessage = "Tournament started successfully.";
      await loadData();
    } catch (e: any) {
      actionError = e.message || "Failed to start tournament";
    } finally {
      tournamentAction = { ...tournamentAction, [tournamentId]: false };
    }
  }
</script>

<div class="page">
  <div class="container">
    <h1>Game Setter Dashboard</h1>

    <div style="display: flex; gap: 0.5rem; flex-wrap: wrap; margin: 0.5rem 0 1rem;">
      <button class="btn btn-secondary" onclick={() => goto("/game-setter/matches")}>
        Match Console
      </button>
      <button class="btn btn-secondary" onclick={loadData}>
        Refresh Data
      </button>
    </div>

    {#if actionMessage}
      <div class="card" style="background: #d1fae5; margin-bottom: 0.75rem;">
        <p class="text-green-700">{actionMessage}</p>
      </div>
    {/if}

    {#if actionError}
      <div class="card" style="background: #fee2e2; margin-bottom: 0.75rem;">
        <p class="text-red-600">{actionError}</p>
      </div>
    {/if}

    {#if error}
      <p class="error-message">{error}</p>
    {/if}

    {#if loading}
      <div class="card text-center">
        <p class="text-gray-500">Loading your games and tournaments...</p>
      </div>
    {:else}
      <div class="grid grid-2">
        <!-- My Games -->
        <div class="card">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
            <h2>My Games</h2>
            <button class="btn btn-primary" onclick={createGame}>Create Game</button>
          </div>

          {#if myGames.length === 0}
            <p class="text-sm">No games yet. Create your first game to get started!</p>
          {:else}
            <div style="display: flex; flex-direction: column; gap: 1rem;">
              {#each myGames as game}
                <div
                  class="card"
                  style="cursor: pointer;"
                  role="button"
                  tabindex="0"
                  onclick={() => manageGame(game.id)}
                  onkeydown={(event) => handleCardKeydown(event, () => manageGame(game.id))}
                >
                  <h3>{game.name}</h3>
                  <p class="text-sm">{game.description}</p>
                  <div style="margin-top: 0.5rem;">
                    <span class="badge {game.is_active ? 'badge-running' : 'badge-cancelled'}">
                      {game.is_active ? 'Active' : 'Inactive'}
                    </span>
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
            <button class="btn btn-primary" onclick={createTournament}>Create Tournament</button>
          </div>

          {#if filteredTournaments.length === 0}
            <p class="text-sm">No tournaments yet for your games.</p>
          {:else}
            <div style="display: flex; flex-direction: column; gap: 1rem;">
              {#each filteredTournaments as tournament}
                <div class="card">
                  <div style="display: flex; justify-content: space-between; gap: 0.75rem;">
                    <div>
                      <h3 style="margin-bottom: 0.35rem;">{tournament.name}</h3>
                      <p class="text-sm">{tournament.description}</p>
                    </div>
                    <span class={getStatusBadge(tournament.status)}>{tournament.status}</span>
                  </div>
                  <div class="text-sm text-gray-600" style="margin: 0.35rem 0 0.75rem;">
                    {tournament.current_players}/{tournament.max_players} players
                  </div>
                  <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
                    <a
                      class="btn btn-secondary"
                      style="padding: 0.4rem 0.9rem;"
                      href="/tournaments/{tournament.id}"
                    >
                      View
                    </a>
                    {#if canOpenRegistration(tournament.status)}
                      <button
                        class="btn btn-primary"
                        style="padding: 0.4rem 0.9rem;"
                        onclick={() => updateTournamentStatus(tournament.id, "registration")}
                        disabled={!!tournamentAction[tournament.id]}
                      >
                        {tournamentAction[tournament.id] ? "Updating..." : "Open Registration"}
                      </button>
                    {/if}
                    {#if canStart(tournament.status)}
                      <button
                        class="btn btn-success"
                        style="padding: 0.4rem 0.9rem;"
                        onclick={() => startTournament(tournament.id)}
                        disabled={!!tournamentAction[tournament.id]}
                      >
                        {tournamentAction[tournament.id] ? "Starting..." : "Start"}
                      </button>
                    {/if}
                    {#if canClose(tournament.status)}
                      <button
                        class="btn btn-secondary"
                        style="padding: 0.4rem 0.9rem;"
                        onclick={() => updateTournamentStatus(tournament.id, "completed")}
                        disabled={!!tournamentAction[tournament.id]}
                      >
                        {tournamentAction[tournament.id] ? "Updating..." : "Close"}
                      </button>
                    {/if}
                    {#if canCancel(tournament.status)}
                      <button
                        class="btn btn-danger"
                        style="padding: 0.4rem 0.9rem;"
                        onclick={() => updateTournamentStatus(tournament.id, "cancelled")}
                        disabled={!!tournamentAction[tournament.id]}
                      >
                        {tournamentAction[tournament.id] ? "Updating..." : "Cancel"}
                      </button>
                    {/if}
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
