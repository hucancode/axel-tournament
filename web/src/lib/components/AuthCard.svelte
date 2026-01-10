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

<main>
    <section class="bg-hatch">
        <h1>{title}</h1>
        {#if error}
            <aside role="alert" class="bg-hatch">
                <p>{error}</p>
            </aside>
        {/if}

        {#if showEmailPassword}
            <form onsubmit={handleSubmit}>
                <fieldset>
                    <label for="email">Email</label>
                    <input
                        type="email"
                        id="email"
                        bind:value={email}
                        required
                        disabled={loading}
                    />
                </fieldset>
                <fieldset>
                    <label for="password">Password</label>
                    <input
                        type="password"
                        id="password"
                        bind:value={password}
                        required
                        disabled={loading}
                    />
                </fieldset>
                <Button
                    variant="primary"
                    label={loading ? "Loading..." : submitText}
                    disabled={loading}
                    type="submit"
                />
            </form>
        {/if}

        {@render children?.()}
    </section>
</main>

<style>
    main {
        max-width: 28rem;
        margin: auto;
        height: 100vh;
        display: flex;
        align-items: center;
    }

    section {
        padding: 1.5rem;
        border: 1px solid var(--blueprint-line-faint);
        width: 100%;
    }

    h1 {
        text-align: center;
        margin-bottom: 1.5rem;
    }

    aside[role="alert"] {
        padding: 1.5rem;
        margin-bottom: 1rem;
        border: 1px solid var(--error);
    }

    aside p {
        color: var(--error);
        margin: 0;
    }

    fieldset {
        border: none;
        padding: 0;
        margin: 0 0 1rem 0;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        color: var(--text-dark);
    }

    input {
        width: 100%;
        padding: 0.75rem;
        border: 1px solid var(--color-border-strong);
        background: var(--color-blueprint-paper);
        font-size: 1rem;
    }

    input:focus {
        outline: 2px solid var(--primary);
        outline-offset: -2px;
    }

    input:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
