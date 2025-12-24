<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { AuthCard } from "$lib";
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

<AuthCard title="Sign Up" {error} {loading}>
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
            <label for="username">Username</label>
            <input
                type="text"
                id="username"
                class="input"
                bind:value={username}
                required
                minlength="3"
                maxlength="50"
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
                minlength="8"
                disabled={loading}
            />
            <p
                class="text-sm text-gray-500"
                style="margin-top: 0.25rem;"
            >
                Minimum 8 characters
            </p>
        </div>
        <div class="form-group">
            <label for="confirmPassword">Confirm Password</label>
            <input
                type="password"
                id="confirmPassword"
                class="input"
                bind:value={confirmPassword}
                required
                disabled={loading}
            />
        </div>
        <div class="form-group">
            <label for="location">Country Code (Optional)</label>
            <input
                type="text"
                id="location"
                class="input"
                bind:value={location}
                placeholder="US, UK, FR, etc."
                maxlength="2"
                disabled={loading}
            />
            <p
                class="text-sm text-gray-500"
                style="margin-top: 0.25rem;"
            >
                2-letter ISO country code
            </p>
        </div>
        <button
            type="submit"
            class="btn btn-primary"
            style="width: 100%;"
            disabled={loading}
        >
            {loading ? "Creating account..." : "Create Account"}
        </button>
    </form>
    <div style="margin-top: 1.5rem; text-align: center;">
        <a href="/login" style="color: var(--primary-600);"
            >Already have an account? Login</a
        >
    </div>
</AuthCard>
