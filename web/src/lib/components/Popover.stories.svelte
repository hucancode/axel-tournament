<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Popover from './Popover.svelte';

  const { Story } = defineMeta({
    title: 'Container/Popover',
    component: Popover,
  });
</script>

<Story name="Default">
  {#snippet template(args)}
    <Popover>
      {#snippet trigger()}
          Click me
      {/snippet}
      {#snippet children()}
        <div class="popover-content">
          <h3>Popover Content</h3>
          <p>This is the content inside the popover.</p>
        </div>
      {/snippet}
    </Popover>
  {/snippet}
</Story>

<Story name="With List">
  {#snippet template()}
    <Popover>
      {#snippet trigger()}
        Open Menu
      {/snippet}
      {#snippet children()}
        <menu class="popover-menu">
          <button class="menu-item">
            Option 1
          </button>
          <button class="menu-item">
            Option 2
          </button>
          <button class="menu-item last">
            Option 3
          </button>
        </menu>
      {/snippet}
    </Popover>
  {/snippet}
</Story>

<style>
  .popover-content {
    padding: 0.5rem;
    min-width: 12.5rem;
  }

  .popover-menu {
    min-width: 12.5rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: 0.75rem 1rem;
    border: none;
    border-bottom: 2px solid black;
    background-color: white;
    text-align: left;
    cursor: pointer;
  }

  .menu-item.last {
    border-bottom: none;
  }

  .menu-item:hover {
    background-color: #f5f5f5;
  }
</style>
