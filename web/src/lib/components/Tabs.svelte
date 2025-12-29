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

<div class={className}>
  <div role="tablist">
    {#each items as item}
      <button
        class:active={activeTab === item.value}
        onclick={() => setActiveTab(item.value)}
        role="tab"
        aria-selected={activeTab === item.value}
        tabindex={activeTab === item.value ? 0 : -1}
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
</div>

<style>
  div {
    width: 100%;
  }
  div[role="tablist"] {
    display: inline-flex;
    align-items: center;
    gap: 0;
    padding: 0;
    background-color: var(--white);
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
    background-color: transparent;
    color: var(--text);
    cursor: pointer;
    transition: border-color 0.15s ease, background-color 0.15s ease;
    white-space: nowrap;
  }

  button:last-child {
    border-right: none;
  }

  button:hover:not(.active) {
    background-color: var(--blueprint-line-faint);
  }

  button:focus {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
  }

  button.active {
    background-color: var(--primary);
    color: var(--white);
    border-color: var(--primary);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  /* Tab content panel */
  div[role="tabpanel"] {
    margin-top: -1px;
    padding: 1.5rem;
    background-color: var(--white);
    border: 1px solid var(--blueprint-line-light);
  }

  div[role="tabpanel"]:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }
</style>
