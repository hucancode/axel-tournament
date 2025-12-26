<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { authStore } from "$lib/stores/auth";
    import { adminService } from "$lib/services/admin";
    import { gameService } from "$lib/services/games";
    import { tournamentService } from "$lib/services/tournaments";
    let loading = $state(true);
    let error = $state("");
    let stats = $state({
        totalUsers: 0,
        totalGames: 0,
        totalTournaments: 0,
        activeTournaments: 0,
    });
    const auth = $derived($authStore);
    onMount(async () => {
        // Check authentication and admin role
        if (!auth.isAuthenticated) {
            goto("/login");
            return;
        }
        if (auth.user?.role !== "admin") {
            goto("/");
            return;
        }
        // Load dashboard stats
        try {
            const [users, games, tournaments] = await Promise.all([
                adminService.listUsers(1, 1000),
                gameService.list(),
                tournamentService.list(),
            ]);
            stats = {
                totalUsers: users.length,
                totalGames: games.length,
                totalTournaments: tournaments.length,
                activeTournaments: tournaments.filter(
                    (t) =>
                        t.status === "running" || t.status === "registration",
                ).length,
            };
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to load dashboard stats";
        } finally {
            loading = false;
        }
    });
</script>

<div class="container page">
    <div class="page-header">
        <h1 class="page-title">Admin Dashboard</h1>
        <p class="text-gray-500">Manage your tournament platform</p>
    </div>
    {#if error}
        <div
            class="card bg-red-100 border-l-4 border-red-600 mb-4"
        >
            <p class="text-red-600">{error}</p>
        </div>
    {/if}
    {#if loading}
        <div class="card text-center">
            <p class="text-gray-500">Loading dashboard...</p>
        </div>
    {:else}
        <!-- Quick Stats -->
        <div class="grid grid-3 mb-4">
            <div class="card">
                <h3 class="text-gray-500 text-sm font-semibold">Total Users</h3>
                <p class="text-2xl font-bold mt-2">{stats.totalUsers}</p>
            </div>
            <div class="card">
                <h3 class="text-gray-500 text-sm font-semibold">Total Games</h3>
                <p class="text-2xl font-bold mt-2">{stats.totalGames}</p>
            </div>
            <div class="card">
                <h3 class="text-gray-500 text-sm font-semibold">
                    Total Tournaments
                </h3>
                <p class="text-2xl font-bold mt-2">{stats.totalTournaments}</p>
            </div>
        </div>
        <div class="card mb-4">
            <h3 class="text-gray-500 text-sm font-semibold">
                Active Tournaments
            </h3>
            <p class="text-2xl font-bold mt-2">{stats.activeTournaments}</p>
        </div>
        <!-- Management Links -->
        <div class="grid grid-2">
            <a
                href="/admin/users"
                class="card no-underline text-inherit transition-transform duration-200 hover:-translate-y-1"
            >
                <h2 class="font-semibold text-lg mb-2">User Management</h2>
                <p class="text-gray-500">View, ban, and manage user accounts</p>
            </a>
            <a
                href="/admin/games"
                class="card no-underline text-inherit transition-transform duration-200 hover:-translate-y-1"
            >
                <h2 class="font-semibold text-lg mb-2">Game Management</h2>
                <p class="text-gray-500">Create, edit, and delete games</p>
            </a>
            <a
                href="/admin/tournaments"
                class="card no-underline text-inherit transition-transform duration-200 hover:-translate-y-1"
            >
                <h2 class="font-semibold text-lg mb-2">
                    Tournament Management
                </h2>
                <p class="text-gray-500">Create and manage tournaments</p>
            </a>
            <a
                href="/admin/matches"
                class="card no-underline text-inherit transition-transform duration-200 hover:-translate-y-1"
            >
                <h2 class="font-semibold text-lg mb-2">Match Management</h2>
                <p class="text-gray-500">View and manage match results</p>
            </a>
        </div>
    {/if}
</div>
