<script lang="ts">
    interface Props {
        message: string;
        type?: "error" | "success" | "warning";
        onclose?: () => void;
    }

    let { message, type = "error", onclose }: Props = $props();
</script>

<div class="card alert alert-{type}">
    <p>{message}</p>
    {#if onclose}
        <button class="close-button" onclick={onclose} aria-label="Close alert">×</button>
    {/if}
</div>

<style>
    .alert {
        margin-bottom: 1rem;
        border-radius: 0;
        border: 1px solid var(--blueprint-line-light);
        position: relative;
        padding-right: 2.5rem;
    }

    .alert-error {
        background: var(--error);
        color: var(--white);
        border-left: 3px solid var(--error);
    }

    .alert-success {
        background: var(--success);
        color: var(--white);
        border-left: 3px solid var(--success);
    }

    .alert-warning {
        background: var(--warning);
        color: var(--blueprint-line);
        border-left: 3px solid var(--warning);
    }

    .alert p {
        margin: 0;
        font-weight: 500;
    }

    .close-button {
        position: absolute;
        top: 50%;
        right: 0.5rem;
        transform: translateY(-50%);
        background: none;
        border: none;
        font-size: 1.5rem;
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
        opacity: 0.7;
        transition: opacity 0.15s ease;
    }

    .close-button:hover {
        opacity: 1;
    }
</style>
