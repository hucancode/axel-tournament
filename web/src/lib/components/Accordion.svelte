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
  <summary class="bg-hatch">
    {title}
  </summary>
  <div>
    {@render children()}
  </div>
</details>

<style>
  details {
    background: var(--blueprint-paper);
  }

  summary {
    padding: 1rem;
    font-weight: 600;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: border-color 0.15s ease;
  }

  summary:hover {
    border-bottom-color: var(--primary);
  }

  details[open] summary {
    border-bottom-color: var(--blueprint-line-faint);
  }

  details > div {
    padding: 1rem;
  }
</style>
