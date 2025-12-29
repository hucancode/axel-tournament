<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    content?: string;
    children?: Snippet;
  }

  let { title = '', content = '', children }: Props = $props();
</script>

<div>
  {#if children}
    {@render children()}
  {:else}
    {#if title}
      <h3>{title}</h3>
    {/if}
    {#if content}
      <p>{content}</p>
    {/if}
  {/if}
</div>

<style>
  div {
    background: var(--white);
    border: 1px solid var(--blueprint-line-light);
    padding: 1.5rem;
  }

  h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }

  p {
    margin: 0;
    color: var(--text-muted);
  }
</style>
