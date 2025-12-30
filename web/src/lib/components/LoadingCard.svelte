<script lang="ts">
    interface Props {
        message?: string;
    }

    let { message = "Loading..." }: Props = $props();
</script>

<article class="border border-gray-800 p-6 bg-hatch text-center" aria-busy="true" aria-live="polite">
    <p class="text-gray-500">{message}</p>
</article>
