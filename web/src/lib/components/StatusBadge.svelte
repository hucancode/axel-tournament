<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    status,
    label,
    title,
    class: className = "",
    children,
  } = $props<{
    status: string;
    label?: string;
    title?: string;
    class?: string;
    children?: Snippet;
  }>();

  let classes = $derived(`badge badge-${status} ${className}`.trim());
</script>

<span class={classes} title={title}>
  {#if children}
    {@render children()}
  {:else}
    {label ?? status}
  {/if}
</span>
