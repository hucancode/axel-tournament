<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    content?: string;
    children?: Snippet;
  }

  let { title = '', content = '', children }: Props = $props();
</script>

<div class="border border-border p-6 shadow-sm bg-hatch">
  {#if children}
    {@render children()}
  {:else}
    {#if title}
      <h3 class="text-lg font-semibold mb-2">{title}</h3>
    {/if}
    {#if content}
      <p class="text-muted">{content}</p>
    {/if}
  {/if}
</div>
