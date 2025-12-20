<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { authService } from "$lib/services/auth";
    import { authStore } from "$lib/stores/auth";
    let error = $state("");
    let loading = $state(true);
    onMount(async () => {
        const code = page.url.searchParams.get("code");
        const state = page.url.searchParams.get("state");
        if (!code || !state) {
            error = "Invalid callback parameters";
            loading = false;
            return;
        }
        try {
            const response = await authService.handleGoogleCallback(
                code,
                state,
            );
            authStore.setAuth(response.user, response.token);
            goto("/");
        } catch (err) {
            error =
                err instanceof Error ? err.message : "Authentication failed";
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
