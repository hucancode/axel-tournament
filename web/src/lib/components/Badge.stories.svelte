<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Badge from './Badge.svelte';

  const { Story } = defineMeta({
    title: 'Data Display/Badge',
    component: Badge,
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>Badge</h2>
      <p class="lede">
        Compact status pill. Each <code>status</code> maps to a colour from the design tokens.
      </p>

      <div class="grid">
        <article class="cell">
          <Badge status="scheduled" label="Scheduled" />
          <code>status="scheduled"</code>
        </article>
        <article class="cell">
          <Badge status="registration" label="Registration Open" />
          <code>status="registration"</code>
        </article>
        <article class="cell">
          <Badge status="generating" label="Generating Matches" />
          <code>status="generating"</code>
        </article>
        <article class="cell">
          <Badge status="running" label="In Progress" />
          <code>status="running"</code>
        </article>
        <article class="cell">
          <Badge status="completed" label="Completed" />
          <code>status="completed"</code>
        </article>
        <article class="cell">
          <Badge status="cancelled" label="Cancelled" />
          <code>status="cancelled"</code>
        </article>
        <article class="cell">
          <Badge status="pending" label="Pending" />
          <code>status="pending"</code>
        </article>
        <article class="cell">
          <Badge status="accepted" label="Accepted" />
          <code>status="accepted"</code>
        </article>
        <article class="cell">
          <Badge status="failed" label="Failed" />
          <code>status="failed"</code>
        </article>
      </div>

      <h3>Inline usage</h3>
      <p>
        Tournament status: <Badge status="running" label="In Progress" />
        — registration still <Badge status="registration" label="Open" />.
      </p>
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
    margin: var(--spacing-6) 0 var(--spacing-2) 0;
    font-size: var(--font-size-md);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-fg-muted);
  }
  .lede {
    color: var(--color-fg-muted);
    margin-bottom: var(--spacing-4);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
    gap: var(--spacing-3);
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    padding: var(--spacing-3);
    background: var(--color-bg-light);
    border: var(--border-thin) solid var(--color-border);
    align-items: flex-start;
  }
  code {
    font-family: monospace;
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
  }
</style>
