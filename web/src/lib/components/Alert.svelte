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
        margin-bottom: 1rem;
        border: 1px solid var(--color-border-strong);
        position: relative;
        padding: 1rem 2.5rem 1rem 1rem;
        border-left-width: 2px;
    }

    aside[data-closable="false"] {
        padding-right: 1rem;
    }

    aside[data-type="error"] {
        background-color: rgb(239 68 68 / 0.1);
        color: var(--color-error);
        border-left-color: var(--color-error);
    }

    aside[data-type="success"] {
        background-color: rgb(16 185 129 / 0.1);
        color: var(--color-success);
        border-left-color: var(--color-success);
    }

    aside[data-type="warning"] {
        background-color: rgb(245 158 11 / 0.1);
        color: #D97706;
        border-left-color: var(--color-warning);
    }

    p {
        margin: 0;
        font-weight: 500;
    }

    button {
        position: absolute;
        top: 50%;
        right: 0.5rem;
        transform: translateY(-50%);
        background-color: transparent;
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
        transition: opacity var(--transition-fast);
    }

    button:hover {
        opacity: 1;
    }
</style>
