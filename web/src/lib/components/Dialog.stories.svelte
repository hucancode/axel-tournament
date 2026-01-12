<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';

  const { Story } = defineMeta({
    title: 'Container/Dialog',
  });
</script>

<Story name="Default">
  {#snippet template()}
    <button onclick={() => document.querySelector("dialog")?.showModal()}>
      Open Dialog
    </button>
    <dialog>
      <form method="dialog">
        <header>
          <h2>Example title</h2>
          <button
            type="button"
            onclick={() => document.querySelector("dialog")?.close()}
            aria-label="Close"
          >×</button>
        </header>
        <div class="dialog-content">
          <p>This is example content in the dialog.</p>
          <input type="text" placeholder="Enter something..." />
        </div>
        <footer>
          <button
            type="button"
            data-variant="secondary"
            onclick={() => document.querySelector("dialog")?.close()}
          >
            Cancel
          </button>
          <button type="submit" data-variant="primary" value="submit">
            Submit
          </button>
        </footer>
      </form>
    </dialog>
  {/snippet}
</Story>
