<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Dialog from './Dialog.svelte';

  const { Story } = defineMeta({
    title: 'Layout Management/Dialog',
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

<Story name="With Form" args={{ title: 'User Information' }}>
  {#snippet template(args)}
    <button class="btn-primary" onclick={() => document.querySelector("dialog")?.showModal()}>
      Open Form Dialog
    </button>
    <Dialog {...args}>
      <div class="form-group">
        <label for="name">Name</label>
        <input id="name" type="text" placeholder="Enter your name" />
      </div>
      <div class="form-group">
        <label for="email">Email</label>
        <input id="email" type="email" placeholder="Enter your email" />
      </div>
    </Dialog>
  {/snippet}
</Story>
