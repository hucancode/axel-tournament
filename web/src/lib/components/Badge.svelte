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
</script>

<span data-status={status} class={className} {title}>
  {#if children}
    {@render children()}
  {:else}
    {label ?? status}
  {/if}
</span>

<style>
  span {
    display: inline-block;
    padding: 0 var(--spacing-2);
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid;
  }

  span[data-status="scheduled"],
  span[data-status="pending"] {
    background: var(--color-bg-popup);
    color: var(--color-fg-dim);
    border-color: var(--color-border);
  }

  span[data-status="registration"] {
    background: var(--color-bg-popup);
    color: var(--color-primary);
    border-color: var(--color-primary);
  }

  span[data-status="generating"] {
    background: var(--color-bg-popup);
    color: var(--color-accent);
    border-color: var(--color-accent);
  }

  span[data-status="running"] {
    background: var(--color-bg-popup);
    color: var(--color-warning);
    border-color: var(--color-warning);
  }

  span[data-status="completed"],
  span[data-status="accepted"] {
    background: var(--color-bg-popup);
    color: var(--color-success);
    border-color: var(--color-success);
  }

  span[data-status="cancelled"],
  span[data-status="failed"] {
    background: var(--color-bg-popup);
    color: var(--color-error);
    border-color: var(--color-error);
  }
</style>
