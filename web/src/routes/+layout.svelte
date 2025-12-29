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
    <title>Axel Tournamentm</title>
</svelte:head>

<div class="min-h-screen flex flex-col bg-blueprint-grid bg-blueprint-major-grid">
    <Nav
        currentPath={currentPath}
        isAuthenticated={isAuthenticated}
        user={user}
        onLogout={logout}
    />
    <main class="flex-1">
        {@render children()}
    </main>
    <footer>
        <div class="max-w-300 mx-auto px-4">
            <p class="text-sm">
                &copy; 2025 Axel Tournament Platform. All rights reserved.
            </p>
        </div>
    </footer>
</div>

<style>
    footer {
        background: var(--color-blueprint-paper);
        color: var(--color-text-muted);
        padding: 2rem 0;
        margin-top: auto;
        text-align: center;
        border-top: 1px solid var(--color-border);
    }

    /* Page layout utilities */
    main :global(.page) {
        min-height: calc(100vh - 8rem);
        padding: 2rem 0;
    }

    main :global(.page-header) {
        margin-bottom: 2rem;
    }

    main :global(.page-title) {
        font-size: 2rem;
        font-weight: bold;
        margin-bottom: 0.5rem;
    }

    main :global(.container) {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1rem;
    }

    main :global(.input),
    main :global(.textarea),
    main :global(.select) {
        width: 100%;
        padding: 0.75rem 1rem;
        border: 1px solid var(--color-border-strong);
        font-size: 1rem;
        background-color: var(--color-blueprint-paper);
        color: var(--color-text);
        transition: border-color 0.15s ease, box-shadow 0.15s ease;
    }

    main :global(.input:focus),
    main :global(.textarea:focus),
    main :global(.select:focus) {
        outline: none;
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgb(59 130 246 / 0.1);
    }

    main :global(.textarea) {
        resize: vertical;
        min-height: 100px;
        font-family: monospace;
    }

    main :global(.form-error) {
        color: var(--color-error);
        font-size: 0.875rem;
        margin-top: 0.25rem;
    }

    /* Table utilities - legacy support (prefer using Table component) */
    main :global(table) {
        width: 100%;
        border-collapse: collapse;
        background: var(--color-blueprint-paper);
        border: 1px solid var(--color-border);
    }

    main :global(th),
    main :global(td) {
        padding: 0.75rem;
        text-align: left;
        border-bottom: 1px solid var(--color-border);
    }

    main :global(th) {
        font-weight: 600;
        background: var(--color-gray-light);
        color: var(--color-text);
        border-bottom: 1px solid var(--color-border-strong);
    }
</style>
