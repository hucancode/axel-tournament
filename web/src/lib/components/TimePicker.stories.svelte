<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import TimePicker from './TimePicker.svelte';

  const { Story } = defineMeta({
    component: TimePicker,
    title: 'Data Entry/TimePicker',
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>TimePicker</h2>
      <p class="lede">
        Hour/minute spinner with AM/PM. Supports <code>min</code>/<code>max</code>, <code>step</code>, <code>disabled</code>, inline <code>error</code>.
      </p>

      <div class="grid">
        <article class="cell">
          <code>default</code>
          <TimePicker label="Select Time" />
        </article>

        <article class="cell">
          <code>with value</code>
          <TimePicker label="Start Time" value="14:30" />
        </article>

        <article class="cell">
          <code>min/max range</code>
          <TimePicker label="Meeting Time" value="10:00" min="09:00" max="17:00" />
        </article>

        <article class="cell">
          <code>15-min step</code>
          <TimePicker label="Appointment Time" value="12:00" step={900} />
        </article>

        <article class="cell">
          <code>disabled</code>
          <TimePicker label="Locked Time" value="15:45" disabled={true} />
        </article>

        <article class="cell">
          <code>with error</code>
          <TimePicker
            label="Deadline Time"
            value="23:59"
            error="This time slot is already booked"
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
