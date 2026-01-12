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
        border: 1px solid var(--color-border);
    }

    data {
        font-size: 1.875rem;
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
        font-size: 0.875rem;
        color: var(--color-fg-dim);
        margin-top: var(--spacing-2);
    }
</style>
