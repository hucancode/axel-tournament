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

<div class="tabs {className}">
  <div class="tabs-list">
    {#each items as item}
      <button
        class="tabs-trigger"
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
    <div class="tabs-content">
      <Component />
    </div>
  {/if}
</div>

<style>
  .tabs {
    width: 100%;
  }

  .tabs-list {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem;
    background-color: var(--gray-light);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
  }

  .tabs-trigger {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem 1.5rem;
    font-weight: 700;
    font-size: 0.875rem;
    border: 2px solid transparent;
    border-radius: 4px;
    background-color: transparent;
    color: var(--black);
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .tabs-trigger:hover:not(.active) {
    background-color: var(--white);
  }

  .tabs-trigger:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  .tabs-trigger.active {
    background-color: var(--primary);
    color: var(--black);
    border-color: var(--black);
    box-shadow: 2px 2px 0 0 var(--black);
  }

  .tabs-trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .tabs-content {
    margin-top: 1rem;
    padding: 1.5rem;
    background-color: var(--white);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 4px 4px 0 0 var(--black);
  }

  .tabs-content:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }
</style>
