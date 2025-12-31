<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { AuthCard, Button } from "$lib/components";
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

<section>
<AuthCard title="Sign Up" {error} {loading}>
    <form onsubmit={handleSubmit}>
        <div class="mb-4">
            <label for="email" class="block mb-2 font-medium text-gray-dark">Email</label>
            <input
                type="email"
                id="email"
                class="input"
                bind:value={email}
                required
                disabled={loading}
            />
        </div>
        <div class="mb-4">
            <label for="username" class="block mb-2 font-medium text-gray-dark">Username</label>
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
        <div class="mb-4">
            <label for="password" class="block mb-2 font-medium text-gray-dark">Password</label>
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
                class="text-sm text-gray-500 mt-1"
            >
                Minimum 8 characters
            </p>
        </div>
        <div class="mb-4">
            <label for="confirmPassword" class="block mb-2 font-medium text-gray-dark">Confirm Password</label>
            <input
                type="password"
                id="confirmPassword"
                class="input"
                bind:value={confirmPassword}
                required
                disabled={loading}
            />
        </div>
        <div class="mb-4">
            <label for="location" class="block mb-2 font-medium text-gray-dark">Country Code (Optional)</label>
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
                class="text-sm text-gray-500 mt-1"
            >
                2-letter ISO country code
            </p>
        </div>
        <Button
            variant="primary"
            type="submit"
            label={loading ? "Creating account..." : "Create Account"}
            disabled={loading}
        />
    </form>
    <div class="mt-6 text-center">
        <a href="/login" class="text-primary-600"
            >Already have an account? Login</a
        >
    </div>
</AuthCard>
</section>
