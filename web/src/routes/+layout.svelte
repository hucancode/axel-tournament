<script lang="ts">
    import "../app.css";
    import { page } from "$app/state";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    let { children } = $props();
    function logout() {
        authStore.logout();
        goto("/login");
    }
    let user = $derived($authStore.user);
    let isAuthenticated = $derived($authStore.isAuthenticated);
    const currentPath = $derived(page.url.pathname);
</script>

<svelte:head>
    <title>Axel Tournament Platform</title>
</svelte:head>

<div class="min-h-screen flex flex-col">
    <nav>
        <div class="nav-content">
            <div class="nav-links">
                <a href="/" class="logo">Axel Tournament</a>
                <a
                    href="/tournaments"
                    class:active={currentPath.startsWith("/tournaments")}
                >
                    Tournaments
                </a>
                <a
                    href="/games"
                    class:active={currentPath.startsWith("/games")}
                >
                    Games
                </a>
                <a
                    href="/leaderboard"
                    class:active={currentPath === "/leaderboard"}
                >
                    Leaderboard
                </a>
            </div>
            <div class="nav-links">
                {#if isAuthenticated}
                    {#if user?.role === "admin"}
                        <a href="/admin">Admin</a>
                    {/if}
                    {#if user?.role === "gamesetter" || user?.role === "admin"}
                        <a href="/game-setter">Game Setter</a>
                    {/if}
                    <a href="/profile">{user?.username}</a>
                    <button onclick={logout} class="btn btn-secondary text-sm"
                        >Logout</button
                    >
                {:else}
                    <a href="/login">Login</a>
                    <a href="/register" class="btn btn-primary text-sm">Sign Up</a>
                {/if}
            </div>
        </div>
    </nav>
    <main class="flex-1">
        {@render children()}
    </main>
    <footer>
        <div class="container">
            <p class="text-sm">
                &copy; 2025 Axel Tournament Platform. All rights reserved.
            </p>
        </div>
    </footer>
</div>
