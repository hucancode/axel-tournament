<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { onMount } from "svelte";
    import type { Tournament } from "$lib/types";
    import { tournamentService } from "$lib/services/tournaments";
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
                class="card text-center"
                style="max-width: 600px; margin: 2rem auto;"
            >
                <h2 class="text-xl font-semibold mb-4">Get Started</h2>
                <p class="mb-4">
                    Create an account to participate in tournaments and submit
                    your code
                </p>
                <div class="flex gap-4" style="justify-content: center;">
                    <a href="/register" class="btn btn-primary">Sign Up</a>
                    <a href="/login" class="btn btn-secondary">Login</a>
                </div>
            </div>
        {:else}
            <div class="card" style="margin-bottom: 2rem;">
                <h2 class="text-xl font-semibold mb-2">
                    Welcome back, {user?.username}!
                </h2>
                <div class="flex gap-4" style="margin-top: 1rem;">
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
                    <a
                        href="/tournaments/{tournament.id}"
                        class="card"
                        style="text-decoration: none; color: inherit; transition: transform 0.2s;"
                        onmouseenter={(e) =>
                            (e.currentTarget.style.transform =
                                "translateY(-4px)")}
                        onmouseleave={(e) =>
                            (e.currentTarget.style.transform = "translateY(0)")}
                    >
                        <h3 class="text-lg font-semibold mb-2">
                            {tournament.name}
                        </h3>
                        <p
                            class="text-sm text-gray-700 mb-4"
                            style="overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;"
                        >
                            {tournament.description}
                        </p>
                        <div class="flex items-center justify-between">
                            <span class="badge badge-{tournament.status}"
                                >{tournament.status}</span
                            >
                            <span class="text-sm text-gray-500">
                                {tournament.current_players}/{tournament.max_players}
                                players
                            </span>
                        </div>
                    </a>
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
