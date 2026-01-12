<script lang="ts">
    import { authService } from "$services/auth";
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

<style>
    main {
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
        padding: var(--spacing-6);
        background-color: var(--color-gray-50);
        margin-bottom: var(--spacing-4);
        color: var(--color-error);
    }

    .success-message {
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

    .form-help {
        font-size: 0.875rem;
        color: var(--color-muted);
        margin-top: var(--spacing-1);
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

<main>
    <div class="container">
        <section class="reset-form-section">
            <h1>Set New Password</h1>
            <p class="form-subtitle">Choose a new password for your account.</p>
            
            {#if !token}
                <div class="error-message">
                    <p>Reset token is missing or invalid.</p>
                </div>
            {/if}
            
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
                    <p class="form-help">Minimum 8 characters</p>
                </div>
                
                <div class="form-field">
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
                    data-variant="primary"
                    disabled={loading || !token}
                >
                    {loading ? "Saving..." : "Reset Password"}
                </button>
            </form>
            
            <div class="back-link">
                <a href="/login">Back to Login</a>
            </div>
        </section>
    </div>
</main>
