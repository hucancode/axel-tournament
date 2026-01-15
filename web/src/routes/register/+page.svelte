<script lang="ts">
    import { authService } from "$services/auth";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";

    let email = $state("");
    let username = $state("");
    let password = $state("");
    let confirmPassword = $state("");
    let location = $state("");
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
        if (password !== confirmPassword) {
            error = "Passwords do not match";
            return;
        }
        if (password.length < 8) {
            error = "Password must be at least 8 characters";
            return;
        }
        loading = true;
        try {
            const response = await authService.register({
                email,
                username,
                password,
                location: location || undefined,
            });
            authStore.setAuth(response.user, response.token);
            goto("/");
        } catch (err) {
            error = err instanceof Error ? err.message : "Registration failed";
        } finally {
            loading = false;
        }
    }
</script>

<main>
    <section class="bg-hatch">
        <h1>Sign Up</h1>
        {#if error}
            <aside role="alert" class="bg-hatch">
                <p>{error}</p>
            </aside>
        {/if}

        <form onsubmit={handleSubmit}>
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
                <label for="username">Username</label>
                <input
                    type="text"
                    id="username"
                    bind:value={username}
                    required
                    minlength="3"
                    maxlength="50"
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
                    minlength="8"
                    disabled={loading}
                />
                <p class="help-text">Minimum 8 characters</p>
            </fieldset>
            <fieldset>
                <label for="confirmPassword">Confirm Password</label>
                <input
                    type="password"
                    id="confirmPassword"
                    bind:value={confirmPassword}
                    required
                    disabled={loading}
                />
            </fieldset>
            <fieldset>
                <label for="location">Country Code (Optional)</label>
                <input
                    type="text"
                    id="location"
                    bind:value={location}
                    placeholder="US, UK, FR, etc."
                    maxlength="2"
                    disabled={loading}
                />
                <p class="help-text">2-letter ISO country code</p>
            </fieldset>
            <button
                type="submit"
                disabled={loading}
                data-variant="primary"
            >
                {loading ? "Creating account..." : "Create Account"}
            </button>
        </form>

        <div class="auth-links">
            <a href="/login">Already have an account? Login</a>
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
        border: 1px solid var(--color-border-light);
        width: 100%;
    }

    h1 {
        text-align: center;
        margin-bottom: 1.5rem;
    }

    aside[role="alert"] {
        padding: 1.5rem;
        margin-bottom: 1rem;
        border: 1px solid var(--color-error);
    }

    aside p {
        color: var(--color-error);
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
        color: var(--color-fg);
    }

    .help-text {
        font-size: 0.875rem;
        color: var(--color-fg-dim);
        margin-top: 0.25rem;
        margin-bottom: 0;
    }
</style>
