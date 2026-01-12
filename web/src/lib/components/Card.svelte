<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        href?: string;
        class?: string;
        children?: Snippet;
    }

    let { href, class: className = "", children }: Props = $props();
    let isLink = $derived(!!href);
</script>

{#if isLink}
    <a class="card {className}" {href}>
        {@render children?.()}
    </a>
{:else}
    <div class="card {className}">
        {@render children?.()}
    </div>
{/if}

<style>
    .card {
        display: block;
        border: 1px solid var(--color-border);
        background: var(--color-bg-light);
        padding: var(--spacing-4);
        text-decoration: none;
        color: inherit;
        transition: border-color var(--transition-fast);
    }

    a.card:hover {
        border-color: var(--color-fg-dim);
    }
</style>
