<script lang="ts">
    import "../app.css";
    import { page } from "$app/state";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { Nav } from "$lib/components";
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

<Nav {currentPath} {isAuthenticated} {user} onLogout={logout} />
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
