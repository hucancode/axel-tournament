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
    <title>Axel Tournament Platform</title>
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
        <div class="container">
            <p class="text-sm">
                &copy; 2025 Axel Tournament Platform. All rights reserved.
            </p>
        </div>
    </footer>
</div>
