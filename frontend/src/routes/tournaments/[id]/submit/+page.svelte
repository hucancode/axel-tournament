<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth';
	import { submissionService } from '$lib/services/submissions';
	import { tournamentService } from '$lib/services/tournaments';
	import type { Tournament, ProgrammingLanguage } from '$lib/types';

	let tournament = $state<Tournament | null>(null);
	let language = $state<ProgrammingLanguage>('rust');
	let code = $state('');
	let loading = $state(false);
	let error = $state('');
	let validationErrors = $state<{ language?: string; code?: string }>({});

	const tournamentId = $derived($page.params.id);
	const auth = $derived($authStore);

	onMount(async () => {
		// Redirect if not authenticated
		if (!auth.isAuthenticated) {
			goto('/login');
			return;
		}

		// Load tournament details
		try {
			tournament = await tournamentService.get(tournamentId);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load tournament';
		}
	});

	function validate(): boolean {
		validationErrors = {};
		let isValid = true;

		if (!language) {
			validationErrors.language = 'Please select a language';
			isValid = false;
		}

		if (!code.trim()) {
			validationErrors.code = 'Please enter your code';
			isValid = false;
		}

		if (code.length > 1000000) {
			validationErrors.code = 'Code must be less than 1MB';
			isValid = false;
		}

		return isValid;
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (!validate()) {
			return;
		}

		loading = true;
		error = '';

		try {
			await submissionService.create({
				tournament_id: tournamentId,
				language,
				code
			});

			// Redirect to tournament page on success
			goto(`/tournaments/${tournamentId}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to submit code';
		} finally {
			loading = false;
		}
	}
</script>

<div class="container page">
	<div class="page-header">
		<h1 class="page-title">Submit Code</h1>
		{#if tournament}
			<p class="text-gray-500">Tournament: {tournament.name}</p>
		{/if}
	</div>

	{#if error}
		<div class="card" style="background-color: #fee2e2; border-left: 4px solid var(--red-600); margin-bottom: 1rem;">
			<p class="text-red-600">{error}</p>
		</div>
	{/if}

	<div class="card">
		<form onsubmit={handleSubmit}>
			<div class="form-group">
				<label for="language">Programming Language</label>
				<select
					id="language"
					class="select"
					bind:value={language}
					disabled={loading}
				>
					<option value="rust">Rust</option>
					<option value="go">Go</option>
					<option value="c">C</option>
				</select>
				{#if validationErrors.language}
					<p class="form-error">{validationErrors.language}</p>
				{/if}
			</div>

			<div class="form-group">
				<label for="code">Code</label>
				<textarea
					id="code"
					class="textarea"
					bind:value={code}
					disabled={loading}
					rows="25"
					placeholder="Paste your code here..."
					style="font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace; font-size: 0.875rem;"
				></textarea>
				{#if validationErrors.code}
					<p class="form-error">{validationErrors.code}</p>
				{/if}
				<p class="text-sm text-gray-500" style="margin-top: 0.5rem;">
					{code.length.toLocaleString()} characters
				</p>
			</div>

			<div class="flex gap-2">
				<button
					type="submit"
					class="btn btn-primary"
					disabled={loading}
				>
					{loading ? 'Submitting...' : 'Submit Code'}
				</button>
				<a href="/tournaments/{tournamentId}" class="btn btn-secondary">
					Cancel
				</a>
			</div>
		</form>
	</div>
</div>
