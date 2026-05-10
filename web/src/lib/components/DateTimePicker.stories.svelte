<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import DateTimePicker from './DateTimePicker.svelte';

  const { Story } = defineMeta({
    component: DateTimePicker,
    title: 'Data Entry/DateTimePicker',
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>DateTimePicker</h2>
      <p class="lede">
        Calendar dropdown + time input. Outputs <code>YYYY-MM-DDTHH:mm</code>. Supports <code>min</code>/<code>max</code>, <code>disabled</code>, inline <code>error</code>.
      </p>

      <div class="grid">
        <article class="cell">
          <code>default</code>
          <DateTimePicker label="Pick date and time" />
        </article>

        <article class="cell">
          <code>with value</code>
          <DateTimePicker label="Tournament Start" value="2024-06-15T18:30" />
        </article>

        <article class="cell">
          <code>min/max range</code>
          <DateTimePicker
            label="Match Window"
            value="2024-06-15T12:00"
            min="2024-06-01T00:00"
            max="2024-06-30T23:59"
          />
        </article>

        <article class="cell">
          <code>disabled</code>
          <DateTimePicker label="Locked" value="2024-03-20T09:00" disabled={true} />
        </article>

        <article class="cell">
          <code>with error</code>
          <DateTimePicker
            label="Deadline"
            value="2024-02-14T23:59"
            error="This slot is already booked"
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
    grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
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
