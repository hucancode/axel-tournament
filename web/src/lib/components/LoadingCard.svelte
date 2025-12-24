<script lang="ts">
    interface Props {
        message?: string;
    }
    
    let { message = "Loading..." }: Props = $props();
</script>

<div class="card text-center">
    <p class="text-gray-500">{message}</p>
</div>
