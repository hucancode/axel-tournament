<script lang="ts">
	import { leaderboardService } from '$lib/services/leaderboard';
	import { tournamentService } from '$lib/services/tournaments';
	import { gameService } from '$lib/services/games';
	import { onMount } from 'svelte';
	import type { LeaderboardEntry, Tournament, Game } from '$lib/types';

	let entries = $state<LeaderboardEntry[]>([]);
	let tournaments = $state<Tournament[]>([]);
	let games = $state<Game[]>([]);
	let loading = $state(true);
	let error = $state('');

	let selectedTournament = $state<string>('all');
	let selectedGame = $state<string>('all');
	let limit = $state(100);

	onMount(async () => {
		await loadFilters();
		await loadLeaderboard();
	});

	async function loadFilters() {
		try {
			// Load tournaments and games for filter dropdowns
			const [tournamentsData, gamesData] = await Promise.all([
				tournamentService.list(),
				gameService.list()
			]);

			tournaments = tournamentsData;
			games = gamesData;
		} catch (err) {
			console.error('Failed to load filters:', err);
		}
	}

	async function loadLeaderboard() {
		loading = true;
		error = '';

		try {
			const filters: { tournament_id?: string; game_id?: string; limit?: number } = {
				limit
			};

			if (selectedTournament !== 'all') {
				filters.tournament_id = selectedTournament;
			}

			if (selectedGame !== 'all') {
				filters.game_id = selectedGame;
			}

			entries = await leaderboardService.get(filters);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load leaderboard';
			console.error('Failed to load leaderboard:', err);
		} finally {
			loading = false;
		}
	}

	async function handleFilterChange() {
		await loadLeaderboard();
	}

	function getMedalEmoji(rank: number): string {
		if (rank === 1) return '🥇';
		if (rank === 2) return '🥈';
		if (rank === 3) return '🥉';
		return '';
	}

	function getRankClass(rank: number): string {
		if (rank === 1) return 'text-rank-1';
		if (rank === 2) return 'text-rank-2';
		if (rank === 3) return 'text-rank-3';
		return '';
	}
</script>

<style>
	.text-rank-1 {
		color: #d97706;
		font-weight: 700;
	}

	.text-rank-2 {
		color: #64748b;
		font-weight: 700;
	}

	.text-rank-3 {
		color: #92400e;
		font-weight: 700;
	}

	.leaderboard-table {
		width: 100%;
		border-collapse: collapse;
		background: white;
	}

	.leaderboard-table thead {
		background: var(--gray-100);
		position: sticky;
		top: 0;
		z-index: 10;
	}

	.leaderboard-table th {
		padding: 0.75rem;
		text-align: left;
		font-weight: 600;
		border-bottom: 2px solid var(--gray-300);
	}

	.leaderboard-table td {
		padding: 0.75rem;
		border-bottom: 1px solid var(--gray-200);
	}

	.leaderboard-table tbody tr:hover {
		background: var(--gray-50);
	}

	.rank-cell {
		font-weight: 700;
		font-size: 1.125rem;
		width: 80px;
	}

	.username-cell {
		font-weight: 600;
		color: var(--gray-900);
	}

	.score-cell {
		font-weight: 700;
		font-size: 1.125rem;
		color: var(--primary-600);
	}

	.filter-section {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1rem;
		margin-bottom: 1.5rem;
	}
</style>

<div class="page">
	<div class="container">
		<div class="page-header">
			<h1 class="page-title">Leaderboard</h1>
			<p class="text-gray-500">Top performing players across all tournaments</p>
		</div>

		<div class="card mb-4">
			<h2 class="text-lg font-semibold mb-4">Filters</h2>

			<div class="filter-section">
				<div class="form-group" style="margin-bottom: 0;">
					<label for="tournament-filter">Tournament</label>
					<select
						id="tournament-filter"
						class="select"
						bind:value={selectedTournament}
						onchange={handleFilterChange}
						disabled={loading}
					>
						<option value="all">All Tournaments</option>
						{#each tournaments as tournament}
							<option value={tournament.id}>{tournament.name}</option>
						{/each}
					</select>
				</div>

				<div class="form-group" style="margin-bottom: 0;">
					<label for="game-filter">Game</label>
					<select
						id="game-filter"
						class="select"
						bind:value={selectedGame}
						onchange={handleFilterChange}
						disabled={loading}
					>
						<option value="all">All Games</option>
						{#each games as game}
							<option value={game.id}>{game.name}</option>
						{/each}
					</select>
				</div>

				<div class="form-group" style="margin-bottom: 0;">
					<label for="limit-filter">Limit</label>
					<select
						id="limit-filter"
						class="select"
						bind:value={limit}
						onchange={handleFilterChange}
						disabled={loading}
					>
						<option value={10}>Top 10</option>
						<option value={25}>Top 25</option>
						<option value={50}>Top 50</option>
						<option value={100}>Top 100</option>
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
				<p class="text-gray-500">Loading leaderboard...</p>
			</div>
		{:else if entries.length === 0}
			<div class="card text-center">
				<p class="text-gray-500">No leaderboard entries found</p>
				<p class="text-sm text-gray-500 mt-2">
					Try adjusting your filters or check back later
				</p>
			</div>
		{:else}
			<div class="card" style="padding: 0; overflow-x: auto;">
				<table class="leaderboard-table">
					<thead>
						<tr>
							<th class="rank-cell">Rank</th>
							<th>Player</th>
							<th>Location</th>
							<th>Score</th>
							<th>Tournament</th>
						</tr>
					</thead>
					<tbody>
						{#each entries as entry, index}
							<tr>
								<td class="rank-cell {getRankClass(entry.rank)}">
									<span style="display: inline-flex; align-items: center; gap: 0.5rem;">
										<span>{entry.rank}</span>
										<span style="font-size: 1.25rem;">{getMedalEmoji(entry.rank)}</span>
									</span>
								</td>
								<td class="username-cell">{entry.username}</td>
								<td>
									{#if entry.location}
										<span class="badge badge-scheduled">{entry.location}</span>
									{:else}
										<span class="text-gray-400">-</span>
									{/if}
								</td>
								<td class="score-cell">{entry.score.toLocaleString()}</td>
								<td>
									<a
										href="/tournaments/{entry.tournament_id}"
										class="text-sm"
										style="color: var(--primary-600); text-decoration: none;"
									>
										{entry.tournament_name}
									</a>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<div class="text-center mt-4 text-sm text-gray-500">
				Showing {entries.length} {entries.length === 1 ? 'entry' : 'entries'}
			</div>
		{/if}
	</div>
</div>
