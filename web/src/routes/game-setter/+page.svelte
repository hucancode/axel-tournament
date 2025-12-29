<script lang="ts">
  import { authStore } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { gameSetterService } from "$lib/services/game-setter";
  import { tournamentService } from "$lib/services/tournaments";
  import type { Game, Tournament, TournamentParticipant, TournamentStatus } from "$lib/types";
  import { Button, LinkButton } from "$lib/components";

  let { user, isAuthenticated } = $derived($authStore);
  let myGames: Game[] = $state([]);
  let allTournaments: Tournament[] = $state([]);
  let participantCounts = $state<Record<string, TournamentParticipant[]>>({});
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

      // Load participants for tournaments related to user's games
      const myGameIds = myGames.map(g => g.id);
      const relevantTournaments = allTournaments.filter(t => myGameIds.includes(t.game_id));

      const participantPromises = relevantTournaments.map(async (tournament) => {
        try {
          const participants = await tournamentService.getParticipants(tournament.id);
          return { tournamentId: tournament.id, participants };
        } catch (err) {
          console.error(`Failed to load participants for tournament ${tournament.id}:`, err);
          return { tournamentId: tournament.id, participants: [] };
        }
      });

      const participantResults = await Promise.all(participantPromises);
      participantCounts = participantResults.reduce((acc, { tournamentId, participants }) => {
        acc[tournamentId] = participants;
        return acc;
      }, {} as Record<string, TournamentParticipant[]>);
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

    <div class="flex gap-2 flex-wrap my-2 mb-4">
      <Button variant="secondary" label="Match Console" onclick={() => goto("/game-setter/matches")} />
      <Button variant="secondary" label="Refresh Data" onclick={loadData} />
    </div>

    {#if actionMessage}
      <div class="border border-[--border-color] p-6 shadow-sm bg-hatch bg-emerald-100 mb-3">
        <p class="text-green-700">{actionMessage}</p>
      </div>
    {/if}

    {#if actionError}
      <div class="border border-[--border-color] p-6 shadow-sm bg-hatch bg-red-100 mb-3">
        <p class="text-red-600">{actionError}</p>
      </div>
    {/if}

    {#if error}
      <p class="error-message">{error}</p>
    {/if}

    {#if loading}
      <div class="border border-[--border-color] p-6 shadow-sm bg-hatch text-center">
        <p class="text-gray-500">Loading your games and tournaments...</p>
      </div>
    {:else}
      <div class="grid grid-2">
        <!-- My Games -->
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
          <div class="flex justify-between items-center mb-4">
            <h2>My Games</h2>
            <Button variant="primary" label="Create Game" onclick={createGame} />
          </div>

          {#if myGames.length === 0}
            <p class="text-sm">No games yet. Create your first game to get started!</p>
          {:else}
            <div class="flex flex-col gap-4">
              {#each myGames as game}
                <div
                  class="border border-[--border-color] p-6 shadow-sm bg-hatch cursor-pointer"
                  role="button"
                  tabindex="0"
                  onclick={() => manageGame(game.id)}
                  onkeydown={(event) => handleCardKeydown(event, () => manageGame(game.id))}
                >
                  <h3>{game.name}</h3>
                  <p class="text-sm">{game.description}</p>
                  <div class="mt-2">
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
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
          <div class="flex justify-between items-center mb-4">
            <h2>My Tournaments</h2>
            <Button variant="primary" label="Create Tournament" onclick={createTournament} />
          </div>

          {#if filteredTournaments.length === 0}
            <p class="text-sm">No tournaments yet for your games.</p>
          {:else}
            <div class="flex flex-col gap-4">
              {#each filteredTournaments as tournament}
                <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
                  <div class="flex justify-between gap-3">
                    <div>
                      <h3 class="mb-1.5">{tournament.name}</h3>
                      <p class="text-sm">{tournament.description}</p>
                    </div>
                    <span class={getStatusBadge(tournament.status)}>{tournament.status}</span>
                  </div>
                  <div class="text-sm text-gray-600 my-1.5 mb-3">
                    {(participantCounts[tournament.id] || []).length}/{tournament.max_players} players
                  </div>
                  <div class="flex gap-2 flex-wrap">
                    <LinkButton
                      variant="secondary"
                      href="/tournaments/{tournament.id}"
                      label="View"
                    />
                    {#if canOpenRegistration(tournament.status)}
                      <Button
                        variant="primary"
                        label={tournamentAction[tournament.id] ? "Updating..." : "Open Registration"}
                        onclick={() => updateTournamentStatus(tournament.id, "registration")}
                        disabled={!!tournamentAction[tournament.id]}
                      />
                    {/if}
                    {#if canStart(tournament.status)}
                      <Button
                        variant="success"
                        label={tournamentAction[tournament.id] ? "Starting..." : "Start"}
                        onclick={() => startTournament(tournament.id)}
                        disabled={!!tournamentAction[tournament.id]}
                      />
                    {/if}
                    {#if canClose(tournament.status)}
                      <Button
                        variant="secondary"
                        label={tournamentAction[tournament.id] ? "Updating..." : "Close"}
                        onclick={() => updateTournamentStatus(tournament.id, "completed")}
                        disabled={!!tournamentAction[tournament.id]}
                      />
                    {/if}
                    {#if canCancel(tournament.status)}
                      <Button
                        variant="danger"
                        label={tournamentAction[tournament.id] ? "Updating..." : "Cancel"}
                        onclick={() => updateTournamentStatus(tournament.id, "cancelled")}
                        disabled={!!tournamentAction[tournament.id]}
                      />
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
