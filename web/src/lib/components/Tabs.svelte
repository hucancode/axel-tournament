<script lang="ts">
  import type { Component } from 'svelte';

  interface TabItem {
    label: string;
    value: number | string;
    component: Component;
  }

  interface Props {
    items: TabItem[];
    class?: string;
  }

  let { items, class: className = '' }: Props = $props();

  let activeTab = $state<number | string | undefined>(undefined);

  $effect(() => {
    if (activeTab === undefined && items.length > 0) {
      activeTab = items[0].value;
    }
  });

  function setActiveTab(value: number | string) {
    activeTab = value;
  }

  let activeComponent = $derived(items.find(item => item.value === activeTab)?.component);
</script>

<section class={className}>
  <div role="tablist">
    {#each items as item}
      <button
        onclick={() => setActiveTab(item.value)}
        role="tab"
        aria-selected={activeTab === item.value}
        tabindex={activeTab === item.value ? 0 : -1}
        data-active={activeTab === item.value}
      >
        {item.label}
      </button>
    {/each}
  </div>

  {#if activeComponent}
    {@const Component = activeComponent}
    <div role="tabpanel">
      <Component />
    </div>
  {/if}
</section>

<style>
  section {
    width: 100%;
  }

  div[role="tablist"] {
    display: inline-flex;
    align-items: center;
    gap: 0;
    padding: 0;
    background: var(--blueprint-paper);
    border: 1px solid var(--blueprint-line-light);
  }

  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem 1.5rem;
    font-weight: 600;
    font-size: 0.875rem;
    border: none;
    border-right: 1px solid var(--blueprint-line-faint);
    background: transparent;
    cursor: pointer;
    transition: background-color 0.15s ease;
    white-space: nowrap;
  }

  button:last-child {
    border-right: none;
  }

  button:hover {
    background: var(--blueprint-line-faint);
  }

  button:focus {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  button[data-active="true"] {
    background: var(--primary);
    color: var(--color-blueprint-paper);
    border-color: var(--primary);
  }

  button[data-active="true"]:hover {
    background: var(--primary);
  }

  div[role="tablist"] {
    display: inline-flex;
    align-items: center;
    gap: 0;
    padding: 0;
    background: var(--blueprint-paper);
    border: 1px solid var(--blueprint-line-light);
  }

  div[role="tabpanel"] {
    margin-top: -1px;
    padding: 1.5rem;
    background: var(--blueprint-paper);
  }

  div[role="tabpanel"]:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }
</style>
