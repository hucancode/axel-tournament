<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { gameService } from "$lib/services/games";
    import { onMount } from "svelte";
    import type { Tournament, Game } from "$lib/types";
    import { tournamentService } from "$lib/services/tournaments";
    import { api } from "$lib/api";
    import TournamentCard from "$lib/components/TournamentCard.svelte";
    import { LinkButton, LoadingCard, EmptyState, Statistic } from "$lib/components";

    let tournaments = $state<Tournament[]>([]);
    let games = $state<Game[]>([]);
    let userCount = $state(0);
    let loading = $state(true);
    let statsLoading = $state(false);

    const user = $derived($authStore.user);
    const isAdmin = $derived($authStore.user?.role === "admin");

    onMount(async () => {
        try {
            tournaments = await tournamentService.list();

            if (isAdmin) {
                await loadAdminStats();
            }
        } catch (err) {
            console.error("Failed to load tournaments:", err);
        } finally {
            loading = false;
        }
    });

    async function loadAdminStats() {
        statsLoading = true;
        try {
            const [gamesData, usersData] = await Promise.all([
                gameService.list(),
                api.get<any[]>("/api/admin/users", true)
            ]);
            games = gamesData;
            userCount = usersData.length;
        } catch (err) {
            console.error("Failed to load admin stats:", err);
        } finally {
            statsLoading = false;
        }
    }
</script>

<div class="container mx-auto">
    <section class="mb-8 text-center">
        <h1 class="text-4xl font-bold mb-2">Welcome to Axel Tournament</h1>
        <p class="text-lg text-gray-700">
            Compete in coding tournaments, submit your AI bots, and climb the
            leaderboard
        </p>
    </section>
    {#if !$authStore.isAuthenticated}
        <section
            class="p-6 bg-hatch text-center max-w-2xl mx-auto my-8"
        >
            <h2 class="text-xl font-semibold mb-4">Get Started</h2>
            <p class="mb-4">
                Create an account to participate in tournaments and submit your
                code
            </p>
            <div class="flex gap-4 justify-center">
                <LinkButton
                    href="/register"
                    variant="primary"
                    label="Sign Up"
                />
                <LinkButton href="/login" variant="secondary" label="Login" />
            </div>
        </section>
    {:else}
        <section
            class="p-6 bg-hatch mb-8"
        >
            <h2 class="text-xl font-semibold mb-2">
                Welcome back, {user?.username}!
            </h2>
            <div class="flex gap-4 mt-4">
                <LinkButton
                    href="/tournaments"
                    variant="primary"
                    label="Browse Tournaments"
                />
                <LinkButton
                    href="/profile"
                    variant="secondary"
                    label="View Profile"
                />
            </div>
        </section>

        {#if isAdmin}
            <section class="mb-8">
                <h2 class="text-2xl font-bold mb-4">Platform Statistics</h2>
                {#if statsLoading}
                    <LoadingCard message="Loading statistics..." />
                {:else}
                    <div class="grid grid-cols-3 gap-4">
                        <Statistic value={userCount} label="Total Users" variant="neutral" />
                        <Statistic value={games.length} label="Total Games" variant="positive" />
                        <Statistic value={tournaments.length} label="Total Tournaments" variant="neutral" />
                    </div>
                {/if}
            </section>
        {/if}
    {/if}
    <section>
        <h2 class="text-2xl font-bold">Active Tournaments</h2>
        {#if loading}
            <LoadingCard message="Loading tournaments..." />
        {:else if tournaments.length === 0}
            <EmptyState message="No active tournaments at the moment. Check back soon!" />
        {:else}
            <div class="grid grid-2">
                {#each tournaments.slice(0, 6) as tournament}
                    <TournamentCard {tournament} />
                {/each}
            </div>
            {#if tournaments.length > 6}
                <div class="text-center mt-4">
                    <LinkButton
                        href="/tournaments"
                        variant="primary"
                        label="View All Tournaments"
                    />
                </div>
            {/if}
        {/if}
    </section>
</div>
