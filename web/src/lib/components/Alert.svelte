<script lang="ts">
    interface Props {
        message: string;
        type?: "error" | "success" | "warning";
        onclose?: () => void;
    }

    let { message, type = "error", onclose }: Props = $props();
</script>

<aside data-type={type} data-closable={!!onclose} role="alert">
    <p>{message}</p>
    {#if onclose}
        <button onclick={onclose} aria-label="Close alert">×</button>
    {/if}
</aside>

<style>
    aside {
        margin-bottom: var(--spacing-4);
        border: 1px solid var(--color-border);
        position: relative;
        padding: var(--spacing-3) var(--spacing-8) var(--spacing-3) var(--spacing-3);
        border-left-width: 3px;
        background: var(--color-bg-light);
    }

    aside[data-closable="false"] {
        padding-right: var(--spacing-3);
    }

    aside[data-type="error"] {
        border-left-color: var(--color-error);
        color: var(--color-error);
    }

    aside[data-type="success"] {
        border-left-color: var(--color-success);
        color: var(--color-success);
    }

    aside[data-type="warning"] {
        border-left-color: var(--color-warning);
        color: var(--color-warning);
    }

    p {
        margin: 0;
        font-weight: 500;
    }

    button {
        position: absolute;
        top: 50%;
        right: var(--spacing-2);
        transform: translateY(-50%);
        background: transparent;
        border: 0;
        font-size: 1.25rem;
        font-weight: 700;
        cursor: pointer;
        padding: 0;
        width: 1.5rem;
        height: 1.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        line-height: 1;
        opacity: 0.6;
        color: inherit;
        transition: opacity var(--transition-fast);
    }

    button:hover {
        opacity: 1;
    }
</style>
