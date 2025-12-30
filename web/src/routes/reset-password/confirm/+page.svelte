<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { page } from "$app/state";
    import { Button } from "$lib/components";

    let token = $derived(page.url.searchParams.get("token") ?? "");
    let password = $state("");
    let confirmPassword = $state("");
    let loading = $state(false);
    let error = $state("");
    let message = $state("");

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        error = "";
        message = "";

        if (!token) {
            error = "Reset token is missing or invalid.";
            return;
        }
        if (password.length < 8) {
            error = "Password must be at least 8 characters";
            return;
        }
        if (password !== confirmPassword) {
            error = "Passwords do not match";
            return;
        }

        loading = true;
        try {
            const response = await authService.confirmReset(token, password);
            message =
                response?.message ||
                "Password reset successful. You can now log in.";
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to reset password";
        } finally {
            loading = false;
        }
    }
</script>

<div class="page">
    <div class="container max-w-md">
        <div class="p-6 bg-hatch">
            <h1 class="page-title text-center">Set New Password</h1>
            <p
                class="text-sm text-gray-500 mb-6 text-center"
            >
                Choose a new password for your account.
            </p>
            {#if !token}
                <div
                    class="p-6 bg-hatch bg-red-100 mb-4"
                >
                    <p class="text-red-600">
                        Reset token is missing or invalid.
                    </p>
                </div>
            {/if}
            {#if error}
                <div
                    class="p-6 bg-hatch bg-red-100 mb-4"
                >
                    <p class="text-red-600">{error}</p>
                </div>
            {/if}
            {#if message}
                <div
                    class="p-6 bg-hatch bg-green-100 mb-4"
                >
                    <p class="text-green-700">{message}</p>
                </div>
            {/if}
            <form onsubmit={handleSubmit}>
                <div class="mb-4">
                    <label for="password" class="block mb-2 font-medium text-gray-dark">New Password</label>
                    <input
                        type="password"
                        id="password"
                        class="input"
                        bind:value={password}
                        required
                        minlength="8"
                        disabled={loading || !token}
                    />
                    <p
                        class="text-sm text-gray-500 mt-1"
                    >
                        Minimum 8 characters
                    </p>
                </div>
                <div class="mb-4">
                    <label for="confirmPassword" class="block mb-2 font-medium text-gray-dark">Confirm New Password</label>
                    <input
                        type="password"
                        id="confirmPassword"
                        class="input"
                        bind:value={confirmPassword}
                        required
                        disabled={loading || !token}
                    />
                </div>
                <Button
                    variant="primary"
                    label={loading ? "Saving..." : "Reset Password"}
                    disabled={loading || !token}
                />
            </form>
            <div class="mt-6 text-center">
                <a href="/login" class="text-primary-600"
                    >Back to Login</a
                >
            </div>
        </div>
    </div>
</div>
