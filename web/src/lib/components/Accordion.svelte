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
    border: 1px solid var(--blueprint-line-light);
    background-color: var(--white);
    overflow: hidden;
  }

  summary {
    padding: 1rem;
    font-weight: 600;
    cursor: pointer;
    background-image: repeating-linear-gradient(315deg, var(--pattern-fg) 0, var(--pattern-fg) 1px, transparent 0, transparent 50%);
    background-size: 8px 8px;
    border-bottom: 0.2rem solid;
    color: var(--text);
  }

  summary {
    border-color: transparent;
  }

  summary:hover {
    border-color: var(--primary);
  }

  details[open] summary {
    border-bottom-color: var(--blueprint-line-faint);
  }

  div {
    padding: 1rem;
  }
</style>
