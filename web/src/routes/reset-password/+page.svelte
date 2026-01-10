<script lang="ts">
    import { authService } from "$services/auth";

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

<style>
    .reset-password-page {
        padding: var(--spacing-8) 0;
    }

    .container {
        max-width: 28rem;
    }

    .reset-form-section {
        padding: var(--spacing-6);
        background-color: var(--color-blueprint-paper);
    }

    .reset-form-section h1 {
        text-align: center;
        margin-bottom: var(--spacing-2);
    }

    .form-subtitle {
        font-size: 0.875rem;
        color: var(--color-muted);
        margin-bottom: var(--spacing-6);
        text-align: center;
    }

    .error-message {
        border: 1px solid var(--color-error);
        padding: var(--spacing-6);
        background-color: var(--color-gray-50);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .success-message {
        border: 1px solid var(--color-success);
        padding: var(--spacing-6);
        background-color: var(--color-gray-50);
        margin-bottom: var(--spacing-4);
        color: var(--color-success);
    }

    .form-field {
        margin-bottom: var(--spacing-4);
    }

    .form-field label {
        display: block;
        margin-bottom: var(--spacing-2);
        font-weight: 500;
        color: var(--color-gray-dark);
    }

    .back-link {
        margin-top: var(--spacing-6);
        text-align: center;
    }

    .back-link a {
        color: var(--color-primary);
        text-decoration: none;
    }
</style>

<main class="reset-password-page">
    <div class="container">
        <section class="reset-form-section">
            <h1>Reset Password</h1>
            <p class="form-subtitle">
                Enter your account email and we'll send you a reset link.
            </p>
            
            {#if error}
                <div class="error-message">
                    <p>{error}</p>
                </div>
            {/if}
            
            {#if message}
                <div class="success-message">
                    <p>{message}</p>
                </div>
            {/if}
            
            <form onsubmit={handleSubmit} class="reset-form">
                <div class="form-field">
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
                    data-variant="primary"
                    disabled={loading}
                >
                    {loading ? "Sending..." : "Send Reset Link"}
                </button>
            </form>
            
            <div class="back-link">
                <a href="/login">Back to Login</a>
            </div>
        </section>
    </div>
</main>
