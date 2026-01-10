<script lang="ts">
    import { authStore } from "$lib/stores/auth";
    import { gameService } from "$services/games";
    import { onMount } from "svelte";
    import type { Tournament, Game } from "$lib/types";
    import { tournamentService } from "$services/tournaments";
    import { api } from "$lib/api";
    import { LinkButton, Statistic, Card, Badge } from "$lib/components";

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

<style>
    .home-page {
        padding: var(--spacing-8) 0;
    }

    .hero-section {
        margin-bottom: var(--spacing-8);
        text-align: center;
    }

    .hero-section h1 {
        font-size: 2.25rem;
        font-weight: bold;
        margin-bottom: var(--spacing-2);
    }

    .hero-subtitle {
        font-size: 1.125rem;
        color: var(--color-muted);
    }

    .cta-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        text-align: center;
        max-width: 32rem;
        margin: var(--spacing-8) auto;
    }

    .cta-section h2 {
        font-size: 1.25rem;
        font-weight: 600;
        margin-bottom: var(--spacing-4);
    }

    .cta-section p {
        margin-bottom: var(--spacing-4);
    }

    .cta-actions {
        display: flex;
        gap: var(--spacing-4);
        justify-content: center;
    }

    .welcome-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        margin-bottom: var(--spacing-8);
    }

    .welcome-section h2 {
        font-size: 1.25rem;
        font-weight: 600;
        margin-bottom: var(--spacing-2);
    }

    .welcome-actions {
        display: flex;
        gap: var(--spacing-4);
        margin-top: var(--spacing-4);
    }

    .stats-section {
        margin-bottom: var(--spacing-8);
    }

    .stats-section h2 {
        font-size: 1.5rem;
        font-weight: bold;
        margin-bottom: var(--spacing-4);
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: var(--spacing-4);
    }

    .tournaments-section h2 {
        font-size: 1.5rem;
        font-weight: bold;
        margin-bottom: var(--spacing-4);
    }

    .tournaments-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: var(--spacing-4);
    }

    .view-all {
        text-align: center;
        margin-top: var(--spacing-4);
    }
</style>

<main class="home-page">
    <div class="container">
        <header class="hero-section">
            <h1>Welcome to Axel Tournament</h1>
            <p class="hero-subtitle">
                Compete in coding tournaments, submit your AI bots, and climb the leaderboard
            </p>
        </header>

        {#if !$authStore.isAuthenticated}
            <section class="cta-section">
                <h2>Get Started</h2>
                <p>Create an account to participate in tournaments and submit your code</p>
                <div class="cta-actions">
                    <LinkButton href="/register" variant="primary" label="Sign Up" />
                    <LinkButton href="/login" variant="secondary" label="Login" />
                </div>
            </section>
        {:else}
            <section class="welcome-section">
                <h2>Welcome back, {user?.username}!</h2>
                <div class="welcome-actions">
                    <LinkButton href="/tournaments" variant="primary" label="Browse Tournaments" />
                    <LinkButton href="/profile" variant="secondary" label="View Profile" />
                </div>
            </section>

            {#if isAdmin}
                <section class="stats-section">
                    <h2>Platform Statistics</h2>
                    {#if statsLoading}
                        <Card class="loading-card">
                            <p>Loading statistics...</p>
                        </Card>
                    {:else}
                        <div class="stats-grid">
                            <Statistic value={userCount} label="Total Users" variant="neutral" />
                            <Statistic value={games.length} label="Total Games" variant="positive" />
                            <Statistic value={tournaments.length} label="Total Tournaments" variant="neutral" />
                        </div>
                    {/if}
                </section>
            {/if}
        {/if}

        <section class="tournaments-section">
            <h2>Active Tournaments</h2>
            {#if loading}
                <Card class="loading-card">
                    <p>Loading tournaments...</p>
                </Card>
            {:else if tournaments.length === 0}
                <Card class="empty-card">
                    <p>No active tournaments at the moment. Check back soon!</p>
                </Card>
            {:else}
                <div class="tournaments-grid">
                    {#each tournaments.slice(0, 6) as tournament}
                        <Card href="/tournaments/tournament?id={tournament.id}">
                            <h3>{tournament.name}</h3>
                            <p>{tournament.description}</p>
                            <footer>
                                <Badge status={tournament.status} label={tournament.status} />
                            </footer>
                        </Card>
                    {/each}
                </div>
                {#if tournaments.length > 6}
                    <div class="view-all">
                        <LinkButton href="/tournaments" variant="primary" label="View All Tournaments" />
                    </div>
                {/if}
            {/if}
        </section>
    </div>
</main>
