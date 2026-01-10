<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        title?: string;
        content?: string;
        href?: string;
        loading?: boolean;
        children?: Snippet;
    }

    let { title = "", content = "", href, loading = false, children }: Props = $props();
    let isLink = $derived(!!href);
</script>

{#if isLink}
    <a class="bg-hatch card-outer" {href} data-loading={loading}>
        <div class="card-inner">
            {#if loading}
                <p>{content || "Loading..."}</p>
            {:else if children}
                {@render children()}
            {:else}
                {#if title}<h3>{title}</h3>{/if}
                {#if content}<p>{content}</p>{/if}
            {/if}
        </div>
    </a>
{:else}
    <article class="bg-hatch card-outer" data-loading={loading}>
        <div class="card-inner">
            {#if loading}
                <p>{content || "Loading..."}</p>
            {:else if children}
                {@render children()}
            {:else}
                {#if title}<h3>{title}</h3>{/if}
                {#if content}<p>{content}</p>{/if}
            {/if}
        </div>
    </article>
{/if}

<style>
    a, article {
        display: block;
        padding: 0.5rem;
    }

    a {
        text-decoration: none;
        transition: border-color var(--transition-fast);
    }

    a:hover {
        border-color: var(--color-border-strong);
    }

    .card-inner {
        border-radius: var(--radius-xl);
        border: 1px solid var(--color-gray-800);
        background-color: var(--color-gray-950);
        padding: 2rem;
    }

    [data-loading="true"] {
        text-align: center;
    }

    [data-loading="true"] p {
        color: var(--color-gray-500);
    }

    h3 {
        font-size: 1.125rem;
        font-weight: 600;
        margin-bottom: 0.5rem;
    }

    p {
        color: var(--color-muted);
    }
</style>
