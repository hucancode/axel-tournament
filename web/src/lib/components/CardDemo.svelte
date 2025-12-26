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
      <h3 style="margin-bottom: 0.5rem;">{title}</h3>
    {/if}
    {#if content}
      <p style="margin: 0; color: #666;">{content}</p>
    {/if}
  {/if}
</div>
