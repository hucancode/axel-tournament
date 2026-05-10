<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Steps from './Steps.svelte';

  const { Story } = defineMeta({
    title: 'Navigation/Steps',
    component: Steps,
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="All Variants">
  {#snippet template()}
    <section class="showcase">
      <h2>Steps</h2>
      <p class="lede">
        Linear progress indicator. <code>current</code> marks the active index. Set <code>clickable</code> for jump navigation.
      </p>

      <div class="stack">
        <article class="cell">
          <code>first step (current=0)</code>
          <Steps
            steps={[
              { label: 'Account Info', description: 'Enter your details' },
              { label: 'Verification', description: 'Verify your email' },
              { label: 'Complete', description: 'All done!' },
            ]}
            current={0}
          />
        </article>

        <article class="cell">
          <code>middle step (current=1)</code>
          <Steps
            steps={[
              { label: 'Account Info', description: 'Enter your details' },
              { label: 'Verification', description: 'Verify your email' },
              { label: 'Complete', description: 'All done!' },
            ]}
            current={1}
          />
        </article>

        <article class="cell">
          <code>completed (current=2)</code>
          <Steps
            steps={[
              { label: 'Account Info', description: 'Enter your details' },
              { label: 'Verification', description: 'Verify your email' },
              { label: 'Complete', description: 'All done!' },
            ]}
            current={2}
          />
        </article>

        <article class="cell">
          <code>label-only steps</code>
          <Steps
            steps={[
              { label: 'Choose Plan' },
              { label: 'Payment' },
              { label: 'Confirm' },
              { label: 'Success' },
            ]}
            current={1}
          />
        </article>

        <article class="cell">
          <code>many steps (6)</code>
          <Steps
            steps={[
              { label: 'Start', description: 'Begin process' },
              { label: 'Details', description: 'Enter information' },
              { label: 'Review', description: 'Check your data' },
              { label: 'Approve', description: 'Confirm submission' },
              { label: 'Submit', description: 'Send to server' },
              { label: 'Complete', description: 'All done!' },
            ]}
            current={2}
          />
        </article>

        <article class="cell">
          <code>clickable</code>
          <Steps
            steps={[
              { label: 'Personal Info', description: 'Name and email' },
              { label: 'Address', description: 'Shipping details' },
              { label: 'Payment', description: 'Card information' },
              { label: 'Review', description: 'Confirm order' },
            ]}
            current={1}
            clickable
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
