<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    class: className = "",
    children,
  } = $props<{
    class?: string;
    children?: Snippet;
  }>();
</script>

<table class={`w-full border-collapse bg-blueprint-paper border border-gray-800 [&_th]:px-3 [&_th]:py-3 [&_th]:text-left [&_th]:font-semibold [&_th]:bg-gray-light [&_th]:border-b [&_th]:border-border-strong [&_td]:px-3 [&_td]:py-3 [&_td]:text-left [&_td]:border-b [&_td]:border-border ${className}`.trim()}>
  {#if children}
    {@render children()}
  {/if}
</table>
