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
    padding: 0.125rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
  }

  span[data-status="scheduled"],
  span[data-status="pending"] {
    background-color: rgb(100 116 139 / 0.1);
    color: var(--color-gray-dark);
  }

  @media (prefers-color-scheme: dark) {
    span[data-status="scheduled"],
    span[data-status="pending"] {
      color: var(--color-gray-400);
    }
  }

  span[data-status="registration"] {
    background-color: rgb(59 130 246 / 0.1);
    color: var(--color-primary);
  }

  span[data-status="generating"] {
    background-color: rgb(236 72 153 / 0.1);
    color: var(--color-accent);
  }

  span[data-status="running"] {
    background-color: rgb(245 158 11 / 0.1);
    color: #D97706;
  }

  span[data-status="completed"],
  span[data-status="accepted"] {
    background-color: rgb(16 185 129 / 0.1);
    color: var(--color-success);
  }

  span[data-status="cancelled"],
  span[data-status="failed"] {
    background-color: rgb(239 68 68 / 0.1);
    color: var(--color-error);
  }
</style>
