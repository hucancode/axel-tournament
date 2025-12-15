<script lang="ts">
    import "../app.css";
    import { page } from "$app/stores";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    let { children } = $props();
    function logout() {
        authStore.logout();
        goto("/login");
    }
    let user = $derived($authStore.user);
    let isAuthenticated = $derived($authStore.isAuthenticated);
</script>

<svelte:head>
    <title>Axel Tournament Platform</title>
</svelte:head>

<div style="min-height: 100vh; display: flex; flex-direction: column;">
    <nav>
        <div class="nav-content">
            <div class="nav-links">
                <a href="/" class="logo">Axel Tournament</a>
                <a
                    href="/tournaments"
                    class:active={$page.url.pathname.startsWith("/tournaments")}
                >
                    Tournaments
                </a>
                <a
                    href="/games"
                    class:active={$page.url.pathname.startsWith("/games")}
                >
                    Games
                </a>
                <a
                    href="/leaderboard"
                    class:active={$page.url.pathname === "/leaderboard"}
                >
                    Leaderboard
                </a>
            </div>
            <div class="nav-links">
                {#if isAuthenticated}
                    {#if user?.role === "admin"}
                        <a href="/admin">Admin</a>
                    {/if}
                    <a href="/profile">{user?.username}</a>
                    <button onclick={logout} class="btn-secondary text-sm"
                        >Logout</button
                    >
                {:else}
                    <a href="/login">Login</a>
                    <a href="/register" class="btn-primary text-sm">Sign Up</a>
                {/if}
            </div>
        </div>
    </nav>
    <main style="flex: 1;">
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
