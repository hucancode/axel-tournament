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
    <button class="btn-primary" onclick={() => document.querySelector("dialog")?.showModal()}>
      Open Dialog
    </button>
    <Dialog {...args}>
      <p>This is example content in the dialog.</p>
      <input type="text" placeholder="Enter something..." />
    </Dialog>
  {/snippet}
</Story>
