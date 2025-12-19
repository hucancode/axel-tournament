<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { authStore } from "$lib/stores/auth";
  import { gameSetterService } from "$lib/services/game-setter";
  import { matchService } from "$lib/services/matches";
  import { tournamentService } from "$lib/services/tournaments";
  import type { Game, Match, MatchStatus, Tournament } from "$lib/types";

  let { user, isAuthenticated } = $derived($authStore);
  let matches: Match[] = $state([]);
  let myGames: Game[] = $state([]);
  let tournaments: Tournament[] = $state([]);
  let loading = $state(true);
  let error = $state("");
  let selectedGame = $state<string>("all");
  let selectedStatus = $state<string>("all");

  // Guard: only game setters or admins
  $effect(() => {
    if (!isAuthenticated || (user?.role !== "gamesetter" && user?.role !== "admin")) {
      goto("/");
    }
  });

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    loading = true;
    error = "";
    try {
      myGames = await gameSetterService.listMyGames();
      tournaments = await tournamentService.list();
      await loadMatches();
    } catch (e: any) {
      error = e.message || "Failed to load match data";
    } finally {
      loading = false;
    }
  }

  async function loadMatches() {
    if (myGames.length === 0) {
      matches = [];
      return;
    }

    const gameIds = selectedGame === "all" ? myGames.map((g) => g.id) : [selectedGame];
    const collected: Match[] = [];

    for (const gameId of gameIds) {
      try {
        const items = await matchService.list({ game_id: gameId });
        collected.push(...items);
      } catch (e: any) {
        console.error(`Failed to load matches for game ${gameId}:`, e);
      }
    }

    const deduped = new Map<string, Match>();
    collected.forEach((m) => {
      deduped.set(m.id, m);
    });

    let filtered = Array.from(deduped.values());
    if (selectedStatus !== "all") {
      filtered = filtered.filter((m) => m.status === selectedStatus);
    }

    matches = filtered.sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    );
  }

  function gameName(gameId: string) {
    return myGames.find((g) => g.id === gameId)?.name || "Unknown game";
  }

  function tournamentName(tournamentId?: string) {
    if (!tournamentId) return "Custom match";
    return tournaments.find((t) => t.id === tournamentId)?.name || "Tournament match";
  }

  function getStatusBadge(status: MatchStatus) {
    return `badge badge-${status}`;
  }

  function formatDate(dateStr?: string) {
    if (!dateStr) return "—";
    return new Date(dateStr).toLocaleString();
  }

  function formatRuntime(match: Match) {
    const meta = match.metadata as Record<string, any> | null;
    const runtimeMs =
      meta?.runtime_ms ??
      meta?.run_time_ms ??
      meta?.duration_ms ??
      meta?.runtime ??
      meta?.duration ??
      meta?.server_runtime_ms ??
      meta?.runner_time_ms ??
      meta?.exec_time_ms ??
      meta?.server_stats?.runtime_ms ??
      meta?.stats?.runtime_ms;

    const runtimeSeconds = meta?.runtime_seconds ?? meta?.duration_seconds;

    if (typeof runtimeMs === "number" && runtimeMs > 0) {
      return runtimeMs >= 1000
        ? `${(runtimeMs / 1000).toFixed(2)} s`
        : `${Math.round(runtimeMs)} ms`;
    }

    if (typeof runtimeSeconds === "number" && runtimeSeconds > 0) {
      return `${runtimeSeconds.toFixed(2)} s`;
    }

    if (match.started_at && match.completed_at) {
      const diff =
        new Date(match.completed_at).getTime() - new Date(match.started_at).getTime();
      if (!Number.isNaN(diff) && diff > 0) {
        return diff >= 1000 ? `${(diff / 1000).toFixed(2)} s` : `${diff} ms`;
      }
    }

    return "N/A";
  }
</script>

<div class="page">
  <div class="container">
    <div class="page-header" style="align-items: center;">
      <div>
        <h1 class="page-title">Match Console</h1>
        <p class="text-gray-500">Monitor matches for your games and tournaments</p>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-secondary" onclick={() => goto("/game-setter")}>Back</button>
        <button class="btn btn-primary" onclick={loadData}>Refresh</button>
      </div>
    </div>

    <div class="card" style="margin-bottom: 1rem;">
      <div class="grid grid-2" style="gap: 1rem;">
        <div class="form-group" style="margin-bottom: 0;">
          <label for="game">Game</label>
          <select
            id="game"
            class="select"
            bind:value={selectedGame}
            onchange={loadMatches}
            disabled={myGames.length === 0}
          >
            <option value="all">All my games</option>
            {#each myGames as game}
              <option value={game.id}>{game.name}</option>
            {/each}
          </select>
        </div>
        <div class="form-group" style="margin-bottom: 0;">
          <label for="status">Status</label>
          <select
            id="status"
            class="select"
            bind:value={selectedStatus}
            onchange={loadMatches}
          >
            <option value="all">Any status</option>
            <option value="pending">Pending</option>
            <option value="queued">Queued</option>
            <option value="running">Running</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </div>
      </div>
    </div>

    {#if error}
      <div class="card" style="background: #fee2e2; margin-bottom: 1rem;">
        <p class="text-red-600">{error}</p>
      </div>
    {/if}

    {#if loading}
      <div class="card text-center">
        <p class="text-gray-500">Loading matches...</p>
      </div>
    {:else if myGames.length === 0}
      <div class="card text-center">
        <p class="text-gray-500">
          You do not have any games yet. Create a game to start scheduling matches.
        </p>
      </div>
    {:else if matches.length === 0}
      <div class="card text-center">
        <p class="text-gray-500">No matches found for the current filters.</p>
      </div>
    {:else}
      <div class="card" style="padding: 0; overflow-x: auto;">
        <table>
          <thead>
            <tr>
              <th>Status</th>
              <th>Game</th>
              <th>Tournament</th>
              <th>Runtime</th>
              <th>Created</th>
              <th>Participants</th>
            </tr>
          </thead>
          <tbody>
            {#each matches as match}
              <tr>
                <td>
                  <span class={getStatusBadge(match.status)}>{match.status}</span>
                </td>
                <td class="font-semibold">{gameName(match.game_id)}</td>
                <td class="text-sm">{tournamentName(match.tournament_id)}</td>
                <td class="text-sm">
                  {formatRuntime(match)}
                  <div class="text-xs text-gray-500">
                    {match.started_at ? `Started ${formatDate(match.started_at)}` : "Not started"}
                  </div>
                  {#if match.completed_at}
                    <div class="text-xs text-gray-500">
                      Completed {formatDate(match.completed_at)}
                    </div>
                  {/if}
                </td>
                <td class="text-sm text-gray-600">
                  <div>Created {formatDate(match.created_at)}</div>
                  <div class="text-xs text-gray-500">Updated {formatDate(match.updated_at)}</div>
                </td>
                <td>
                  <div class="text-xs text-gray-700" style="display: flex; gap: 0.35rem; flex-wrap: wrap;">
                    {#each match.participants as participant}
                      <span class="badge badge-scheduled">
                        {participant.score ?? 0}
                      </span>
                    {/each}
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
