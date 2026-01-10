<script lang="ts">
    import "$styles/variables.css";
    import "$styles/base.css";
    import "$styles/forms.css";
    import "$styles/buttons.css";
    import "$styles/tabs.css";
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
    <div class="nav-container">
        <div class="nav-links">
            <a href="/" class="brand">Axel Tournament</a>
            <a
                href="/tournaments"
                class:active={currentPath.startsWith('/tournaments')}
            >
                Tournaments
            </a>
            <a
                href="/games"
                class:active={currentPath.startsWith('/games')}
            >
                Games
            </a>
            <a
                href="/rooms"
                class:active={currentPath.startsWith('/rooms')}
            >
                Rooms
            </a>
            <a
                href="/leaderboard"
                class:active={currentPath === '/leaderboard'}
            >
                Leaderboard
            </a>
        </div>
        <div class="nav-actions">
            {#if isAuthenticated}
                <a href="/profile" class:active={currentPath === '/profile'}>{user?.username}</a>
                <button onclick={logout} data-variant="ghost">Logout</button>
            {:else}
                <a href="/login" class:active={currentPath === '/login'}>Login</a>
                <LinkButton href="/register" variant="primary" label="Sign Up" />
            {/if}
        </div>
    </div>
</nav>
<div class="page-wrapper bg-blueprint-fine-grid bg-blueprint-major-grid">
    <div class="bg-hatch sidebar"></div>
    <main>
        {@render children()}
    </main>
    <div class="bg-hatch sidebar"></div>
</div>
<footer>
    <div class="footer-content">
        <p>
            &copy; 2025 Axel Tournament Platform. All rights reserved.
        </p>
    </div>
</footer>

<style>
    nav {
        border-bottom: 1px solid var(--color-blueprint-line-light);
        background-color: var(--color-blueprint-paper);
        padding: 1rem 2rem;
    }

    .nav-container {
        max-width: 75rem;
        margin-inline: auto;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .nav-links,
    .nav-actions {
        display: flex;
        gap: 1.5rem;
        align-items: center;
    }

    nav a {
        text-decoration: none;
        font-weight: 500;
        color: inherit;
        transition: color var(--transition-fast), opacity var(--transition-fast);
    }

    nav a:hover {
        color: var(--color-primary);
    }

    nav a.active {
        color: var(--color-primary);
        font-weight: 600;
    }

    nav a:focus-visible {
        outline: 2px solid var(--color-primary);
        outline-offset: 2px;
    }

    .brand {
        font-size: 1.25rem;
        font-weight: 700;
        color: var(--color-primary);
    }

    .brand:hover {
        opacity: 0.8;
    }

    .page-wrapper {
        display: flex;
        flex: 1;
        justify-content: center;
    }

    .sidebar {
        width: 2rem;
        height: 100%;
    }

    main {
        flex: 1;
        max-width: 1200px;
        background-color: var(--color-gray-950);
    }

    footer {
        background: var(--color-blueprint-paper);
        padding: 2rem 0;
        margin-top: auto;
        text-align: center;
        border-top: 1px solid var(--color-border);
    }

    .footer-content {
        max-width: 75rem;
        margin-inline: auto;
        padding-inline: 1rem;
    }

    .footer-content p {
        font-size: 0.875rem;
        margin: 0;
    }
</style>
