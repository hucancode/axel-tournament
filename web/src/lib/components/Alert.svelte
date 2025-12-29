<script lang="ts">
    interface Props {
        message: string;
        type?: "error" | "success" | "warning";
        onclose?: () => void;
    }

    let { message, type = "error", onclose }: Props = $props();
</script>

<aside class="alert" class:error={type === 'error'} class:success={type === 'success'} class:warning={type === 'warning'} role="alert">
    <p>{message}</p>
    <button hidden={!onclose} onclick={onclose} aria-label="Close alert">×</button>
</aside>

<style>
    aside {
        margin-bottom: 1rem;
        border: 1px solid var(--border-color-strong);
        position: relative;
        padding: 1rem 2.5rem 1rem 1rem;
    }

    .error {
        background: rgb(239 68 68 / 0.1);
        color: var(--error);
        border-left: 2px solid var(--error);
    }

    .success {
        background: rgb(16 185 129 / 0.1);
        color: var(--success);
        border-left: 2px solid var(--success);
    }

    .warning {
        background: rgb(245 158 11 / 0.1);
        color: #D97706;
        border-left: 2px solid var(--warning);
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
        background: none;
        border: none;
        font-size: 1.25rem;
        font-weight: bold;
        cursor: pointer;
        padding: 0;
        width: 1.5rem;
        height: 1.5rem;
        display: flex;
        align-items: center;
        justify-content: center;
        line-height: 1;
        color: inherit;
        opacity: 0.6;
        transition: opacity 0.15s ease;
    }

    button:hover {
        opacity: 1;
    }
</style>
