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

<article class="border border-border p-6 shadow-sm bg-hatch text-center" role="status" aria-live="polite">
    {#if illustration}
        <div class="mb-4">
            {@render illustration()}
        </div>
    {:else if illustrationSrc}
        <div class="mb-4 flex justify-center">
            <img src={illustrationSrc} alt={illustrationAlt} class="max-w-48 max-h-48 opacity-50" />
        </div>
    {/if}

    <p class="text-gray-500 mb-4">{message}</p>

    {#if actionLabel && onAction}
        <div class="flex gap-2 justify-center">
            <Button label={actionLabel} variant={actionVariant} onclick={onAction} />
        </div>
    {/if}
</article>
