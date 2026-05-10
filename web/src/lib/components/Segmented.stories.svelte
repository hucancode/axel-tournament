<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Segmented from './Segmented.svelte';

  const { Story } = defineMeta({
    title: 'Data Entry/Segmented',
    component: Segmented,
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>Segmented</h2>
      <p class="lede">
        Inline radio group. Pass <code>options</code> as <code>{`{ value, label }[]`}</code>.
      </p>

      <div class="stack">
        <article class="cell">
          <code>three options (default)</code>
          <Segmented
            options={[
              { value: 'list', label: 'List' },
              { value: 'grid', label: 'Grid' },
              { value: 'table', label: 'Table' },
            ]}
            value="list"
          />
        </article>

        <article class="cell">
          <code>two options</code>
          <Segmented
            options={[
              { value: 'light', label: 'Light' },
              { value: 'dark', label: 'Dark' },
            ]}
            value="light"
          />
        </article>

        <article class="cell">
          <code>four options</code>
          <Segmented
            options={[
              { value: 'all', label: 'All' },
              { value: 'active', label: 'Active' },
              { value: 'pending', label: 'Pending' },
              { value: 'completed', label: 'Completed' },
            ]}
            value="all"
          />
        </article>

        <article class="cell">
          <code>disabled</code>
          <Segmented
            options={[
              { value: 'option1', label: 'Option 1' },
              { value: 'option2', label: 'Option 2' },
              { value: 'option3', label: 'Option 3' },
            ]}
            value="option1"
            disabled
          />
        </article>
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
  .lede {
    color: var(--color-fg-muted);
    margin-bottom: var(--spacing-4);
  }
  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-4);
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    align-items: flex-start;
  }
  code {
    font-family: monospace;
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
  }
</style>
