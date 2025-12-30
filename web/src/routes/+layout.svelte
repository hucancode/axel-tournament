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
<main class="bg-blueprint-fine-grid bg-blueprint-major-grid">
    {@render children()}
</main>
<footer>
    <div class="max-w-300 mx-auto px-4">
        <p class="text-sm">
            &copy; 2025 Axel Tournament Platform. All rights reserved.
        </p>
    </div>
</footer>
