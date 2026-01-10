<script lang="ts">
    import type { Snippet } from "svelte";
    import Button from "./Button.svelte";

    interface Props {
        message?: string;
        illustration?: Snippet;
        illustrationSrc?: string;
        illustrationAlt?: string;
        actionLabel?: string;
        onAction?: () => void;
        actionVariant?: "primary" | "secondary" | "success" | "danger" | "ghost";
    }

    let {
        message = "No items found",
        illustration,
        illustrationSrc,
        illustrationAlt = "Empty state",
        actionLabel,
        onAction,
        actionVariant = "primary"
    }: Props = $props();
</script>

<article role="status" aria-live="polite" class="bg-hatch">
    {#if illustration}
        <figure>
            {@render illustration()}
        </figure>
    {:else if illustrationSrc}
        <figure>
            <img src={illustrationSrc} alt={illustrationAlt} />
        </figure>
    {/if}

    <p>{message}</p>

    {#if actionLabel && onAction}
        <footer>
            <button onclick={onAction}>{actionLabel}</button>
        </footer>
    {/if}
</article>

<style>
    article {
        border: 1px solid var(--color-gray-800);
        padding: 1.5rem;
        text-align: center;
    }

    figure {
        margin: 0 0 1rem 0;
        display: flex;
        justify-content: center;
    }

    img {
        max-width: 12rem;
        max-height: 12rem;
        opacity: 0.5;
    }

    p {
        color: var(--color-gray-500);
        margin: 0 0 1rem 0;
    }

    footer {
        display: flex;
        gap: 0.5rem;
        justify-content: center;
    }
</style>
