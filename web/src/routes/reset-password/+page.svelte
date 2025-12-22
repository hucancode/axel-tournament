<script lang="ts">
    import { authService } from "$lib/services/auth";

    let email = $state("");
    let loading = $state(false);
    let error = $state("");
    let message = $state("");

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        error = "";
        message = "";
        loading = true;
        try {
            const response = await authService.resetPassword(email);
            message =
                response?.message ||
                "If the email exists, a reset link has been sent.";
        } catch (err) {
            error =
                err instanceof Error
                    ? err.message
                    : "Failed to request password reset";
        } finally {
            loading = false;
        }
    }
</script>

<div class="page">
    <div class="container" style="max-width: 420px;">
        <div class="card">
            <h1 class="page-title text-center">Reset Password</h1>
            <p
                class="text-sm text-gray-500"
                style="margin-bottom: 1.5rem; text-align: center;"
            >
                Enter your account email and we'll send you a reset link.
            </p>
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
                <button
                    type="submit"
                    class="btn btn-primary"
                    style="width: 100%;"
                    disabled={loading}
                >
                    {loading ? "Sending..." : "Send Reset Link"}
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
