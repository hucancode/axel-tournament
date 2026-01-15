<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        title: string;
        subtitle?: string;
        children?: Snippet;
    }

    let { title, subtitle, children }: Props = $props();
</script>

<header>
    <div class="content">
        <div class="title">
            <h1>{title}</h1>
            {#if subtitle}
                <p class="subtitle">{subtitle}</p>
            {/if}
        </div>
        {#if children}
            <div class="actions">
                {@render children()}
            </div>
        {/if}
    </div>
</header>

<style>
    header {
        margin-bottom: var(--spacing-8);
    }

    .content {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: var(--spacing-4);
    }

    .title {
        flex: 1;
        min-width: 0; /* Allow text truncation if needed */
    }

    .title h1 {
        font-size: 2rem;
        margin: 0;
    }

    .subtitle {
        color: var(--color-fg-muted);
        margin: var(--spacing-1) 0 0 0;
        font-size: 0.875rem;
    }

    .actions {
        display: flex;
        gap: var(--spacing-2);
        align-items: center;
        flex-shrink: 0;
    }

    /* Responsive: Stack on smaller screens */
    @media (max-width: 640px) {
        .content {
            flex-direction: column;
            align-items: flex-start;
        }

        .actions {
            width: 100%;
        }
    }
</style>
