<script lang="ts">
    import { authService } from "$services/auth";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { env } from "$env/dynamic/public";

    let email = $state("");
    let password = $state("");
    let error = $state("");
    let loading = $state(false);

    onMount(() => {
        if ($authStore.isAuthenticated) {
            goto("/");
        }
    });

    async function handleLogin(e: SubmitEvent) {
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

    function handleGoogleLogin() {
        const apiUrl = env.PUBLIC_API_URL || "http://localhost:8080";
        window.location.href = `${apiUrl}/api/auth/google`;
    }
</script>

<main>
    <section class="bg-hatch">
        <h1>Login</h1>
        {#if error}
            <aside role="alert" class="bg-hatch">
                <p>{error}</p>
            </aside>
        {/if}

        <form onsubmit={handleLogin}>
            <fieldset>
                <label for="email">Email</label>
                <input
                    type="email"
                    id="email"
                    bind:value={email}
                    required
                    disabled={loading}
                />
            </fieldset>
            <fieldset>
                <label for="password">Password</label>
                <input
                    type="password"
                    id="password"
                    bind:value={password}
                    required
                    disabled={loading}
                />
            </fieldset>
            <button
                type="submit"
                disabled={loading}
                data-variant="primary"
            >
                {loading ? "Loading..." : "Login"}
            </button>
        </form>

        <div class="divider">or</div>
        <button onclick={handleGoogleLogin} data-variant="secondary">
            Continue with Google
        </button>
        <div class="auth-links">
            <a href="/register">Don't have an account? Sign up</a>
        </div>
        <div class="forgot-link">
            <a href="/reset-password">Forgot password?</a>
        </div>
    </section>
</main>

<style>
    main {
        max-width: 28rem;
        margin: auto;
        height: 100vh;
        display: flex;
        align-items: center;
    }

    section {
        padding: 1.5rem;
        border: 1px solid var(--blueprint-line-faint);
        width: 100%;
    }

    h1 {
        text-align: center;
        margin-bottom: 1.5rem;
    }

    aside[role="alert"] {
        padding: 1.5rem;
        margin-bottom: 1rem;
        border: 1px solid var(--error);
    }

    aside p {
        color: var(--error);
        margin: 0;
    }

    fieldset {
        border: none;
        padding: 0;
        margin: 0 0 1rem 0;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        color: var(--text-dark);
    }

    .divider {
        margin: var(--spacing-6) 0;
        text-align: center;
        color: var(--color-muted);
    }

    .auth-links {
        margin-top: var(--spacing-6);
        text-align: center;
    }

    .auth-links a {
        color: var(--color-primary);
        text-decoration: none;
    }

    .forgot-link {
        margin-top: var(--spacing-2);
        text-align: center;
    }

    .forgot-link a {
        color: var(--color-primary);
        text-decoration: none;
        font-size: 0.875rem;
    }
</style>
