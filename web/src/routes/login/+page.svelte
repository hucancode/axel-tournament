<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { authStore } from "$lib/stores/auth";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { env } from "$env/dynamic/public";
    import { AuthCard, Button } from "$lib/components";
    let error = $state("");
    let loading = $state(false);
    onMount(() => {
        if ($authStore.isAuthenticated) {
            goto("/");
        }
    });
    async function handleLogin({ email, password }: { email: string; password: string }) {
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
        // Navigate directly to backend OAuth endpoint
        const apiUrl = env.PUBLIC_API_URL || "http://localhost:8080";
        window.location.href = `${apiUrl}/api/auth/google`;
    }
</script>

<section>
<AuthCard
    title="Login"
    {error}
    {loading}
    showEmailPassword={true}
    submitText="Login"
    onsubmit={handleLogin}
>
    <div
        class="my-6 text-center text-gray-500"
    >
        or
    </div>
    <Button
        onclick={handleGoogleLogin}
        variant="secondary"
        label="Continue with Google"
    />
    <div class="mt-6 text-center">
        <a href="/register" class="text-primary-600"
            >Don't have an account? Sign up</a
        >
    </div>
    <div class="mt-2 text-center">
        <a
            href="/reset-password"
            class="text-primary-600 text-sm"
            >Forgot password?</a
        >
    </div>
</AuthCard>
</section>
