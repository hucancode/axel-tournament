<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        title: string;
        onclose?: () => void;
        dialog?: HTMLDialogElement | null;
        children?: Snippet;
    }
    let {
        title,
        onclose = () => {},
        dialog = $bindable(null),
        children,
    }: Props = $props();
</script>

<dialog bind:this={dialog} {onclose}>
    <form method="dialog">
        <header>
            <h2>{title}</h2>
            <button
                type="button"
                onclick={() => dialog?.close()}
                aria-label="Close"
            >×</button>
        </header>
        {#if children}
            <div class="dialog-content">
                {@render children()}
            </div>
        {/if}
        <footer>
            <button
                type="button"
                data-variant="secondary"
                onclick={() => dialog?.close()}
            >
                Cancel
            </button>
            <button type="submit" data-variant="primary" value="submit">
                Submit
            </button>
        </footer>
    </form>
</dialog>

<style>
    dialog {
        max-width: 31.25rem;
        width: 90%;
        background-color: var(--color-blueprint-paper);
        color: var(--color-fg);
        border: 1px solid var(--color-border-strong);
        padding: 0;
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    dialog::backdrop {
        background-color: rgb(15 23 42 / 0.6);
    }

    form {
        display: flex;
        flex-direction: column;
    }

    header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.5rem;
        border-bottom: 1px solid var(--color-blueprint-line-faint);
    }

    h2 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 600;
    }
    .dialog-content {
        padding: 1.5rem;
    }

    footer {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
        padding: 1.5rem;
        border-top: 1px solid var(--color-blueprint-line-faint);
    }
</style>
