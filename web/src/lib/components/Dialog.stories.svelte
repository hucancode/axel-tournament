<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Dialog from './Dialog.svelte';

  const { Story } = defineMeta({
    title: 'Container/Dialog',
    component: Dialog,
  });
</script>

<Story name="Default" args={{title: "Example title"}}>
  {#snippet template(args)}
    <button class="open-button" onclick={() => document.querySelector("dialog")?.showModal()}>
      Open Dialog
    </button>
    <Dialog {...args}>
      <p>This is example content in the dialog.</p>
      <input type="text" placeholder="Enter something..." />
    </Dialog>
  {/snippet}
</Story>

<style>
  .open-button {
    padding: 0.75rem 1.5rem;
    background-color: #3b82f6;
    color: white;
    border: 2px solid black;
    border-radius: 0;
    font-weight: bold;
    cursor: pointer;
    box-shadow: 4px 4px 0 black;
    transition: all 0.1s ease;
  }

  .open-button:hover {
    transform: translate(2px, 2px);
    box-shadow: 2px 2px 0 black;
  }

  .open-button:active {
    transform: translate(4px, 4px);
    box-shadow: none;
  }
</style>
