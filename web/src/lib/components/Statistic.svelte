<script lang="ts">
    type Variant = "neutral" | "positive" | "negative";

    interface Props {
        value: number | string;
        label: string;
        variant?: Variant;
    }

    let { value, label, variant = "neutral" }: Props = $props();
</script>

<article data-variant={variant}>
    <data value={typeof value === 'number' ? value : undefined}>
        {value}
    </data>
    <footer>{label}</footer>
</article>

<style>
    article {
        padding: var(--spacing-4);
        text-align: center;
        background: var(--color-bg-light);
        border: var(--border-thin) solid var(--color-border);
    }

    data {
        font-size: var(--font-size-3xl);
        font-weight: bold;
        display: block;
        color: var(--color-primary);
    }

    article[data-variant="positive"] data {
        color: var(--color-success);
    }

    article[data-variant="negative"] data {
        color: var(--color-error);
    }

    footer {
        font-size: var(--font-size-sm);
        color: var(--color-fg-dim);
        margin-top: var(--spacing-2);
    }
</style>
