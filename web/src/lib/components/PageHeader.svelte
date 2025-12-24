<script lang="ts">
    interface Props {
        title: string;
        subtitle?: string;
        centered?: boolean;
    }
    
    let { title, subtitle, centered = false }: Props = $props();
</script>

<div class="page-header" class:text-center={centered}>
    <h1 class="page-title">{title}</h1>
    {#if subtitle}
        <p class="text-lg text-gray-700">{subtitle}</p>
    {/if}
</div>
