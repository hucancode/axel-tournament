<script lang="ts">
  import { type Snippet } from 'svelte';

  interface Props {
    title: string;
    open?: boolean;
    children: Snippet;
  }

  let { title, open = false, children }: Props = $props();
</script>

<details {open} class="bg-blueprint-paper">
  <summary class="p-4 font-semibold cursor-pointer bg-hatch border-b-2 border-transparent hover:border-primary">
    {title}
  </summary>
  <div class="p-4">
    {@render children()}
  </div>
</details>

<style>
  details[open] summary {
    border-bottom-color: var(--blueprint-line-faint);
  }
</style>
