<script lang="ts">
  interface StepItem {
    label: string;
    description?: string;
  }

  interface Props {
    steps: StepItem[];
    current?: number;
    clickable?: boolean;
    onchange?: (index: number) => void;
  }

  let { steps, current = 0, clickable = false, onchange }: Props = $props();

  function handleStepClick(index: number) {
    if (!clickable) return;
    if (onchange) {
      onchange(index);
    }
  }

  function getStepStatus(index: number): 'completed' | 'current' | 'upcoming' {
    if (index < current) return 'completed';
    if (index === current) return 'current';
    return 'upcoming';
  }
</script>

<div role="navigation" aria-label="Progress steps">
  {#each steps as step, index (index)}
    <button
      type="button"
      data-status={getStepStatus(index)}
      data-clickable={clickable || undefined}
      disabled={!clickable}
      onclick={() => handleStepClick(index)}
      aria-current={getStepStatus(index) === 'current' ? 'step' : undefined}
    >
      <div>
        {#if getStepStatus(index) === 'completed'}
          <svg class="icon"><use href="/icons.svg#i-check" /></svg>
        {:else}
          <span>{index + 1}</span>
        {/if}
      </div>
      <div>
        <div>{step.label}</div>
        {#if step.description}
          <div>{step.description}</div>
        {/if}
      </div>
    </button>
    {#if index < steps.length - 1}
      <div
        data-connector
        data-completed={index < current || undefined}
      ></div>
    {/if}
  {/each}
</div>

<style>
  div[role="navigation"] {
    display: flex;
    width: 100%;
    align-items: center;
  }

  button {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
    padding: var(--spacing-3);
    background: var(--color-bg-light);
    border: var(--border-thin) solid var(--color-border);
    cursor: default;
    transition: border-color var(--transition-fast);
    flex: 1 1 0;
    min-width: 0;
    color: var(--color-fg);
  }

  button[data-clickable] {
    cursor: pointer;
  }

  button[data-clickable]:hover {
    border-color: var(--color-primary);
  }

  button[data-status="current"] {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-bg-dark);
  }

  button[data-status="upcoming"] {
    background: var(--color-bg-popup);
    opacity: 0.7;
  }

  button > div:first-child {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    flex-shrink: 0;
    border: var(--border-thin) solid var(--color-border);
    background: var(--color-bg-light);
    font-weight: 600;
    font-size: var(--font-size-sm);
  }

  button[data-status="completed"] > div:first-child {
    background: var(--color-success);
    border-color: var(--color-success);
    color: var(--color-bg-dark);
  }

  button[data-status="current"] > div:first-child {
    background: var(--color-bg-light);
    border-color: var(--color-bg-light);
    color: var(--color-primary);
  }

  button[data-status="upcoming"] > div:first-child {
    background: var(--color-bg-popup);
    color: var(--color-fg-dim);
  }

  svg {
    --icon-size: 1rem;
  }

  button > div:first-child > span {
    line-height: 1;
  }

  button > div:last-child {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    flex: 1;
  }

  button > div:last-child > div:first-child {
    font-weight: 600;
    font-size: var(--font-size-sm);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  button > div:last-child > div:last-child {
    font-size: var(--font-size-xs);
    color: var(--color-fg-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  button[data-status="current"] > div:last-child > div:last-child {
    color: rgba(255, 255, 255, 0.8);
  }

  button[data-status="upcoming"] > div:last-child > div {
    opacity: 0.6;
  }

  div[data-connector] {
    height: 1px;
    flex: 0 0 1rem;
    background: var(--color-border);
    margin: 0 var(--spacing-2);
  }

  div[data-connector][data-completed] {
    background: var(--color-success);
  }

  @media (max-width: 768px) {
    div[role="navigation"] {
      flex-direction: column;
      align-items: stretch;
      gap: var(--spacing-2);
    }

    button {
      flex: none;
    }

    div[data-connector] {
      flex: none;
      width: 1px;
      height: 1rem;
      margin: 0 0 0 var(--spacing-4);
    }

    button > div:last-child > div {
      white-space: normal;
    }
  }
</style>
