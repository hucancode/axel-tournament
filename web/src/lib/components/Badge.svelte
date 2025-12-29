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

  const statusClasses: Record<string, string> = {
    scheduled: "bg-slate-500/10 text-gray-dark dark:text-slate-400",
    registration: "bg-primary/10 text-primary",
    generating: "bg-accent/10 text-accent",
    running: "bg-amber-500/10 text-amber-600",
    completed: "bg-success/10 text-success",
    cancelled: "bg-error/10 text-error",
    pending: "bg-slate-500/10 text-gray-dark dark:text-slate-400",
    accepted: "bg-success/10 text-success",
    failed: "bg-error/10 text-error",
  };

  let classes = $derived(`inline-block px-2.5 py-0.5 text-xs font-medium ${statusClasses[status] || ""} ${className}`.trim());
</script>

<span class={classes} title={title}>
  {#if children}
    {@render children()}
  {:else}
    {label ?? status}
  {/if}
</span>
