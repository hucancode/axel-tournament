<script lang="ts">
    import { Button } from "$lib/components";

    interface LoginData {
        email: string;
        password: string;
    }

    let {
        title,
        error = "",
        loading = false,
        showEmailPassword = false,
        submitText = "Submit",
        onsubmit,
        children
    }: {
        title: string;
        error?: string;
        loading?: boolean;
        showEmailPassword?: boolean;
        submitText?: string;
        onsubmit?: (data: LoginData) => void;
        children?: any;
    } = $props();

    let email = $state("");
    let password = $state("");

    function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        onsubmit?.({ email, password });
    }
</script>

<div class="page">
    <div class="container max-w-md">
        <div class="card">
            <h1 class="page-title text-center">{title}</h1>
            {#if error}
                <div class="card bg-red-100 mb-4">
                    <p class="text-red-600">{error}</p>
                </div>
            {/if}

            {#if showEmailPassword}
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
                        <label for="password">Password</label>
                        <input
                            type="password"
                            id="password"
                            class="input"
                            bind:value={password}
                            required
                            disabled={loading}
                        />
                    </div>
                    <Button
                        variant="primary"
                        label={loading ? "Loading..." : submitText}
                        disabled={loading}
                    />
                </form>
            {/if}

            {@render children?.()}
        </div>
    </div>
</div>
