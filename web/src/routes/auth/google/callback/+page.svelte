<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { authStore } from "$lib/stores/auth";
    import { authService } from "$lib/services/auth";
    let error = $state("");
    let loading = $state(true);
    onMount(async () => {
        const token = page.url.searchParams.get("token");
        if (!token) {
            error = "Missing authentication token";
            loading = false;
            return;
        }
        try {
            // Store token temporarily
            localStorage.setItem("auth_token", token);
            // Fetch user profile
            const user = await authService.getProfile();
            authStore.setAuth(user, token);
            goto("/");
        } catch (err) {
            error =
                err instanceof Error ? err.message : "Failed to fetch user profile";
            loading = false;
        }
    });
</script>

<div class="page">
    <div class="container text-center">
        {#if loading}
            <h1>Authenticating with Google...</h1>
        {:else if error}
            <div class="card" style="max-width: 500px; margin: 0 auto;">
                <h1 class="text-2xl font-bold mb-4">Authentication Failed</h1>
                <p class="text-red-600 mb-4">{error}</p>
                <a href="/login" class="btn btn-primary">Back to Login</a>
            </div>
        {/if}
    </div>
</div>
