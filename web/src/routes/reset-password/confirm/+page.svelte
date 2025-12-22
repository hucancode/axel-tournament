<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { page } from "$app/state";

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
    <div class="container" style="max-width: 420px;">
        <div class="card">
            <h1 class="page-title text-center">Set New Password</h1>
            <p
                class="text-sm text-gray-500"
                style="margin-bottom: 1.5rem; text-align: center;"
            >
                Choose a new password for your account.
            </p>
            {#if !token}
                <div
                    class="card"
                    style="background: #fee2e2; margin-bottom: 1rem;"
                >
                    <p class="text-red-600">
                        Reset token is missing or invalid.
                    </p>
                </div>
            {/if}
            {#if error}
                <div
                    class="card"
                    style="background: #fee2e2; margin-bottom: 1rem;"
                >
                    <p class="text-red-600">{error}</p>
                </div>
            {/if}
            {#if message}
                <div
                    class="card"
                    style="background: #d1fae5; margin-bottom: 1rem;"
                >
                    <p class="text-green-700">{message}</p>
                </div>
            {/if}
            <form onsubmit={handleSubmit}>
                <div class="form-group">
                    <label for="password">New Password</label>
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
                        class="text-sm text-gray-500"
                        style="margin-top: 0.25rem;"
                    >
                        Minimum 8 characters
                    </p>
                </div>
                <div class="form-group">
                    <label for="confirmPassword">Confirm New Password</label>
                    <input
                        type="password"
                        id="confirmPassword"
                        class="input"
                        bind:value={confirmPassword}
                        required
                        disabled={loading || !token}
                    />
                </div>
                <button
                    type="submit"
                    class="btn btn-primary"
                    style="width: 100%;"
                    disabled={loading || !token}
                >
                    {loading ? "Saving..." : "Reset Password"}
                </button>
            </form>
            <div style="margin-top: 1.5rem; text-align: center;">
                <a href="/login" style="color: var(--primary-600);"
                    >Back to Login</a
                >
            </div>
        </div>
    </div>
</div>
