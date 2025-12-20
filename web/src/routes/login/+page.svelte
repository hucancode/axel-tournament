<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    let email = $state("");
    let password = $state("");
    let error = $state("");
    let loading = $state(false);
    onMount(() => {
        if ($authStore.isAuthenticated) {
            goto("/");
        }
    });
    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        error = "";
        loading = true;
        try {
            const response = await authService.login({ email, password });
            authStore.setAuth(response.user, response.token);
            goto("/");
        } catch (err) {
            error = err instanceof Error ? err.message : "Login failed";
        } finally {
            loading = false;
        }
    }
    async function handleGoogleLogin() {
        try {
            const { url } = await authService.getGoogleAuthUrl();
            window.location.href = url;
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to initiate Google login";
        }
    }
</script>

<div class="page">
    <div class="container" style="max-width: 400px;">
        <div class="card">
            <h1 class="page-title text-center">Login</h1>
            {#if error}
                <div
                    class="card"
                    style="background: #fee2e2; margin-bottom: 1rem;"
                >
                    <p class="text-red-600">{error}</p>
                </div>
            {/if}
            <form onsubmit={handleSubmit}>
                <div class="form-group">
                    <label for="email">Email</label>
                    <input
                        type="email"
                        id="email"
                        class="input"
                        bind:value={email}
                        required
                        disabled={loading}
                    />
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input
                        type="password"
                        id="password"
                        class="input"
                        bind:value={password}
                        required
                        disabled={loading}
                    />
                </div>
                <button
                    type="submit"
                    class="btn btn-primary"
                    style="width: 100%;"
                    disabled={loading}
                >
                    {loading ? "Logging in..." : "Login"}
                </button>
            </form>
            <div
                style="margin: 1.5rem 0; text-align: center; color: var(--gray-500);"
            >
                or
            </div>
            <button
                onclick={handleGoogleLogin}
                class="btn btn-secondary"
                style="width: 100%;"
            >
                Continue with Google
            </button>
            <div style="margin-top: 1.5rem; text-align: center;">
                <a href="/register" style="color: var(--primary-600);"
                    >Don't have an account? Sign up</a
                >
            </div>
            <div style="margin-top: 0.5rem; text-align: center;">
                <a
                    href="/reset-password"
                    style="color: var(--primary-600); font-size: 0.875rem;"
                    >Forgot password?</a
                >
            </div>
        </div>
    </div>
</div>
