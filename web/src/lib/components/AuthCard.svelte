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
        children,
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

<div class="container max-w-md mx-auto my-auto">
    <div class="p-6 bg-hatch border-blueprint-line-faint">
        <h1 class="page-title text-center">{title}</h1>
        {#if error}
            <div class="bg-red-100 p-6 shadow-sm bg-hatch mb-4">
                <p class="text-red-600">{error}</p>
            </div>
        {/if}

        {#if showEmailPassword}
            <form onsubmit={handleSubmit}>
                <div class="mb-4">
                    <label
                        for="email"
                        class="block mb-2 font-medium text-gray-dark"
                        >Email</label
                    >
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
                    <label
                        for="password"
                        class="block mb-2 font-medium text-gray-dark"
                        >Password</label
                    >
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
