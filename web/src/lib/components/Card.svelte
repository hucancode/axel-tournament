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
    <a class="bg-hatch card {className}" {href}>
        {@render children?.()}
    </a>
{:else}
    <div class="bg-hatch card {className}">
        {@render children?.()}
    </div>
{/if}

<style>
    .card {
        display: block;
        border: 1px solid var(--color-gray-800);
        padding: 1.5rem;
        text-decoration: none;
        transition: border-color 0.15s ease;
    }

    a.card:hover {
        border-color: var(--border-strong);
    }
</style>
