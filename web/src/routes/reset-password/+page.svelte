<script lang="ts">
    import { authService } from "$lib/services/auth";
    import { Button } from "$lib/components";

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
    <div class="container max-w-md">
        <div class="border border-[--border-color] p-6 shadow-sm bg-hatch">
            <h1 class="page-title text-center">Reset Password</h1>
            <p
                class="text-sm text-gray-500 mb-6 text-center"
            >
                Enter your account email and we'll send you a reset link.
            </p>
            {#if error}
                <div
                    class="border p-6 shadow-sm bg-hatch bg-red-100 mb-4"
                >
                    <p class="text-red-600">{error}</p>
                </div>
            {/if}
            {#if message}
                <div
                    class="border p-6 shadow-sm bg-hatch bg-green-100 mb-4"
                >
                    <p class="text-green-700">{message}</p>
                </div>
            {/if}
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
                <Button
                    variant="primary"
                    label={loading ? "Sending..." : "Send Reset Link"}
                    disabled={loading}
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
