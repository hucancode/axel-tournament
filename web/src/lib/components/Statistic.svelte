<script lang="ts">
    type Variant = "neutral" | "positive" | "negative";

    interface Props {
        value: number | string;
        label: string;
        variant?: Variant;
    }

    let { value, label, variant = "neutral" }: Props = $props();
</script>

<article data-variant={variant} class="bg-hatch">
    <data value={typeof value === 'number' ? value : undefined}>
        {value}
    </data>
    <footer>{label}</footer>
</article>

<style>
    article {
        padding: 1.5rem;
        text-align: center;
    }

    data {
        font-size: 1.875rem;
        font-weight: bold;
        display: block;
        color: var(--primary);
    }

    article[data-variant="positive"] data {
        color: var(--green-600);
    }

    article[data-variant="negative"] data {
        color: var(--red-600);
    }

    footer {
        font-size: 0.875rem;
        color: var(--color-gray-600);
        margin-top: 0.5rem;
    }
</style>
