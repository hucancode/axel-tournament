<script lang="ts">
    interface Props {
        message?: string;
    }

    let { message = "Loading..." }: Props = $props();
</script>

<article aria-busy="true" aria-live="polite" class="bg-hatch">
    <p>{message}</p>
</article>

<style>
    article {
        border: 1px solid var(--color-gray-800);
        padding: 1.5rem;
        text-align: center;
    }

    p {
        color: var(--color-gray-500);
        margin: 0;
    }
</style>
