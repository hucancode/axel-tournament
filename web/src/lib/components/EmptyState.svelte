<script lang="ts">
    interface Props {
        message?: string;
    }

    let { message = "No items found" }: Props = $props();
</script>

<article class="card text-center" role="status" aria-live="polite">
    <p class="text-gray-500">{message}</p>
</article>
