<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { onMount } from "svelte";
    import type { Tournament } from "$lib/types";
    import { tournamentService } from "$lib/services/tournaments";
    import TournamentCard from "$lib/components/TournamentCard.svelte";
    let tournaments = $state<Tournament[]>([]);
    let loading = $state(true);
    onMount(async () => {
        try {
            tournaments = await tournamentService.list();
        } catch (err) {
            console.error("Failed to load tournaments:", err);
        } finally {
            loading = false;
        }
    });
    let user = $derived($authStore.user);
</script>

<div class="page">
    <div class="container">
        <div class="page-header text-center">
            <h1 class="page-title">Welcome to Axel Tournament</h1>
            <p class="text-lg text-gray-700">
                Compete in coding tournaments, submit your AI bots, and climb
                the leaderboard
            </p>
        </div>
        {#if !$authStore.isAuthenticated}
            <div
                class="card text-center max-w-2xl mx-auto my-8"
            >
                <h2 class="text-xl font-semibold mb-4">Get Started</h2>
                <p class="mb-4">
                    Create an account to participate in tournaments and submit
                    your code
                </p>
                <div class="flex gap-4 justify-center">
                    <a href="/register" class="btn btn-primary">Sign Up</a>
                    <a href="/login" class="btn btn-secondary">Login</a>
                </div>
            </div>
        {:else}
            <div class="card mb-8">
                <h2 class="text-xl font-semibold mb-2">
                    Welcome back, {user?.username}!
                </h2>
                <div class="flex gap-4 mt-4">
                    <a href="/tournaments" class="btn btn-primary"
                        >Browse Tournaments</a
                    >
                    <a href="/profile" class="btn btn-secondary">View Profile</a>
                </div>
            </div>
        {/if}
        <div class="page-header">
            <h2 class="text-2xl font-bold">Active Tournaments</h2>
        </div>
        {#if loading}
            <p>Loading tournaments...</p>
        {:else if tournaments.length === 0}
            <div class="card text-center">
                <p>No active tournaments at the moment. Check back soon!</p>
            </div>
        {:else}
            <div class="grid grid-2">
                {#each tournaments.slice(0, 6) as tournament}
                    <TournamentCard tournament={tournament} />
                {/each}
            </div>
            {#if tournaments.length > 6}
                <div class="text-center mt-4">
                    <a href="/tournaments" class="btn btn-primary"
                        >View All Tournaments</a
                    >
                </div>
            {/if}
        {/if}
    </div>
</div>
