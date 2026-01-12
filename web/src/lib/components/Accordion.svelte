<script lang="ts">
  import { type Snippet } from 'svelte';

  interface Props {
    title: string;
    open?: boolean;
    children: Snippet;
  }

  let { title, open = false, children }: Props = $props();
</script>

<details {open}>
  <summary>
    {title}
  </summary>
  <div>
    {@render children()}
  </div>
</details>

<style>
  details {
    background: var(--color-bg-light);
    border: 1px solid var(--color-border);
  }

  summary {
    padding: var(--spacing-3);
    font-weight: 600;
    cursor: pointer;
    border-bottom: 1px solid transparent;
    transition: border-color var(--transition-fast);
    color: var(--color-fg);
  }

  summary:hover {
    border-bottom-color: var(--color-primary);
  }

  details[open] summary {
    border-bottom-color: var(--color-border-light);
  }

  details > div {
    padding: var(--spacing-3);
  }
</style>
