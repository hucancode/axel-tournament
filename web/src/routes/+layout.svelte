<script lang="ts">
    import "$styles/variables.css";
    import "$styles/base.css";
    import "$styles/forms.css";
    import "$styles/buttons.css";
    import "$styles/tabs.css";
    import "$styles/dialog.css";
    import { page } from "$app/state";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { LinkButton } from "$lib/components";
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
    <title>Axel Tournament</title>
</svelte:head>

<nav>
    <a href="/">Axel Tournament</a>
    <a href="/tournaments" class:active={currentPath.startsWith("/tournaments")}
        >Tournaments</a
    >
    <a href="/games" class:active={currentPath.startsWith("/games")}>Games</a>
    <a href="/rooms" class:active={currentPath.startsWith("/rooms")}>Rooms</a>
    <a href="/leaderboard" class:active={currentPath === "/leaderboard"}
        >Leaderboard</a
    >
    <span></span>
    {#if isAuthenticated}
        <a href="/profile" class:active={currentPath === "/profile"}
            >{user?.username}</a
        >
        <button onclick={logout} data-variant="ghost">Logout</button>
    {:else}
        <a href="/login" class:active={currentPath === "/login"}>Login</a>
        <LinkButton href="/register" variant="primary" label="Sign Up" />
    {/if}
</nav>
<div class="bg-grid bg-grid-major">
    {@render children()}
</div>
<footer>&copy; 2025 Axel Tournament Platform</footer>

<style>
    nav {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        padding: var(--spacing-3) var(--spacing-6);
        background: var(--color-bg-light);
        border-bottom: 1px solid var(--color-border-light);
    }

    nav a {
        text-decoration: none;
        color: var(--color-fg-muted);
        font-weight: 500;
        transition: color var(--transition-fast);
    }

    nav a:first-child {
        font-weight: 700;
        color: var(--color-primary);
        margin-right: var(--spacing-4);
    }

    nav a:hover {
        color: var(--color-fg);
    }

    nav a.active {
        color: var(--color-primary);
    }

    nav span {
        flex: 1;
    }

    div {
        flex: 1;
        display: flex;
        justify-content: center;
    }

    footer {
        padding: var(--spacing-4);
        text-align: center;
        background: var(--color-bg-light);
        border-top: 1px solid var(--color-border-light);
        font-size: 0.875rem;
        color: var(--color-fg-dim);
    }
</style>
