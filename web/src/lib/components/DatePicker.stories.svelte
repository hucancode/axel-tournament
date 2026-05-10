<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import DatePicker from './DatePicker.svelte';

  const { Story } = defineMeta({
    component: DatePicker,
    title: 'Data Entry/DatePicker',
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>DatePicker</h2>
      <p class="lede">
        Calendar dropdown. Supports <code>min</code>/<code>max</code>, <code>disabled</code>, and inline <code>error</code>.
      </p>

      <div class="grid">
        <article class="cell">
          <code>default</code>
          <DatePicker label="Select Date" />
        </article>

        <article class="cell">
          <code>with value</code>
          <DatePicker label="Birth Date" value="2024-01-15" />
        </article>

        <article class="cell">
          <code>min/max range</code>
          <DatePicker
            label="Tournament Date"
            value="2024-06-15"
            min="2024-01-01"
            max="2024-12-31"
          />
        </article>

        <article class="cell">
          <code>disabled</code>
          <DatePicker label="Locked Date" value="2024-03-20" disabled={true} />
        </article>

        <article class="cell">
          <code>with error</code>
          <DatePicker
            label="Event Date"
            value="2024-02-14"
            error="This date is not available"
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
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    gap: var(--spacing-4);
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    padding: var(--spacing-3);
    background: var(--color-bg-light);
    border: var(--border-thin) solid var(--color-border);
  }
  code {
    font-family: monospace;
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
  }
</style>
