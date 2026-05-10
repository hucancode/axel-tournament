<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Alert from './Alert.svelte';

  const { Story } = defineMeta({
    title: 'Data Display/Alert',
    component: Alert,
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>Alert</h2>
      <p class="lede">
        Inline notification. <code>type</code> tints the left border. Pass <code>onclose</code> for a dismiss button.
      </p>

      <h3>Types</h3>
      <div class="stack">
        <div class="cell">
          <code>type="error"</code>
          <Alert message="Operation failed. Please try again." type="error" />
        </div>
        <div class="cell">
          <code>type="success"</code>
          <Alert message="Operation completed successfully." type="success" />
        </div>
        <div class="cell">
          <code>type="warning"</code>
          <Alert message="Please check your input." type="warning" />
        </div>
      </div>

      <h3>Closable</h3>
      <div class="stack">
        <div class="cell">
          <code>with onclose</code>
          <Alert message="Click the × to dismiss." type="warning" onclose={() => {}} />
        </div>
      </div>

      <h3>Long message</h3>
      <div class="stack">
        <div class="cell">
          <Alert
            type="error"
            message="Tournament generation failed: 3 of 16 players had invalid bots. Re-upload the bots and retry generation. Contact support if the issue persists."
          />
        </div>
      </div>
    </section>
  {/snippet}
</Story>

<style>
  .showcase {
    padding: var(--spacing-4);
    max-width: 60rem;
  }
  h2 {
    margin: 0 0 var(--spacing-2) 0;
  }
  h3 {
    margin: var(--spacing-6) 0 var(--spacing-3) 0;
    font-size: var(--font-size-md);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-fg-muted);
  }
  .lede {
    color: var(--color-fg-muted);
    margin-bottom: var(--spacing-4);
  }
  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }
  code {
    font-family: monospace;
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
  }
</style>
