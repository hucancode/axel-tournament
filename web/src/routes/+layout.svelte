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
<div class="bg-blueprint-fine-grid bg-blueprint-major-grid flex grow justify-center">
    <div class="bg-hatch h-full w-8"></div>
    <main class="container bg-gray-950">
        {@render children()}
    </main>
    <div class="bg-hatch h-full w-8"></div>
</div>
<footer>
    <div class="max-w-300 mx-auto px-4">
        <p class="text-sm">
            &copy; 2025 Axel Tournament Platform. All rights reserved.
        </p>
    </div>
</footer>
