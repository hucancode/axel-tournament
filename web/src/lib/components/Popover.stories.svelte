<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Popover from './Popover.svelte';
    import Button from './Button.svelte';

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
        <div class="p-2 min-w-50">
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
        <div class="min-w-50">
          <button class="block w-full py-3 px-4 border-none border-b-2 border-black bg-white text-left cursor-pointer">
            Option 1
          </button>
          <button class="block w-full py-3 px-4 border-none border-b-2 border-black bg-white text-left cursor-pointer">
            Option 2
          </button>
          <button class="block w-full py-3 px-4 border-none bg-white text-left cursor-pointer">
            Option 3
          </button>
        </div>
      {/snippet}
    </Popover>
  {/snippet}
</Story>
