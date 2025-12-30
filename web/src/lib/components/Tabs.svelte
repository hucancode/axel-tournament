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

<div class="w-full {className}">
  <div role="tablist" class="inline-flex items-center gap-0 p-0 bg-blueprint-paper border border-blueprint-line-light">
    {#each items as item}
      <button
        class="inline-flex items-center justify-center px-6 py-3 font-semibold text-sm border-0 border-r border-blueprint-line-faint bg-transparent cursor-pointer transition-all whitespace-nowrap last:border-r-0 hover:bg-blueprint-line-faint focus:outline-2 focus:outline-primary focus:-outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none {activeTab === item.value ? 'bg-primary text-white border-primary hover:bg-primary' : ''}"
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
    <div role="tabpanel" class="-mt-px p-6 bg-blueprint-paper focus:outline-2 focus:outline-primary focus:outline-offset-2">
      <Component />
    </div>
  {/if}
</div>
