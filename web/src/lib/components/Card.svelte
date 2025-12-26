<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    content?: string;
    children?: Snippet;
  }

  let { title = '', content = '', children }: Props = $props();
</script>

<div class="card">
  {#if children}
    {@render children()}
  {:else}
    {#if title}
      <h3 class="mb-2">{title}</h3>
    {/if}
    {#if content}
      <p class="m-0 text-gray-600">{content}</p>
    {/if}
  {/if}
</div>
