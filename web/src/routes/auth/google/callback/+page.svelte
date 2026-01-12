<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { authStore } from "$lib/stores/auth";
    import { authService } from "$services/auth";
    import { LinkButton } from "$lib/components";
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

<style>
    main {
        padding: var(--spacing-8) 0;
    }

    .loading-section, .error-section {
        text-align: center;
    }

    .error-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
        max-width: 32rem;
        margin: 0 auto;
    }

    .error-section h1 {
        font-size: 1.5rem;
        font-weight: bold;
        margin-bottom: var(--spacing-4);
    }

    .error-message {
        color: var(--color-error);
        margin-bottom: var(--spacing-4);
    }
</style>

<main>
    <div class="container">
        {#if loading}
            <section class="loading-section">
                <h1>Authenticating with Google...</h1>
            </section>
        {:else if error}
            <section class="error-section">
                <h1>Authentication Failed</h1>
                <p class="error-message">{error}</p>
                <LinkButton href="/login" variant="primary" label="Back to Login" />
            </section>
        {/if}
    </div>
</main>
