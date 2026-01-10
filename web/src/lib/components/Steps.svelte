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
    <div>
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
            <svg
              viewBox="0 0 20 20"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M16.6667 5L7.50004 14.1667L3.33337 10"
                stroke="currentColor"
                stroke-width="3"
                stroke-linecap="square"
                stroke-linejoin="miter"
              />
            </svg>
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
    </div>
  {/each}
</div>

<style>
  /* Root navigation */
  div[role="navigation"] {
    display: flex;
    width: 100%;
    gap: 0;
  }

  /* Step wrapper - first level div */
  div[role="navigation"] > div {
    display: flex;
    flex: 1;
    align-items: center;
    min-width: 0;
  }

  /* Step button */
  button {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background-color: var(--color-blueprint-paper);
    border: 1px solid var(--blueprint-line-light);
    cursor: default;
    transition: border-color 0.15s ease;
    flex: 1;
    min-width: 0;
  }

  button[data-clickable] {
    cursor: pointer;
  }

  button[data-clickable]:hover {
    border-color: var(--primary);
    border-width: 2px;
    padding: calc(1rem - 1px);
  }

  button[data-clickable]:active {
    opacity: 0.9;
  }

  button:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
    z-index: 1;
  }

  button[data-status="current"] {
    background-color: var(--primary);
    border-color: var(--primary);
    color: var(--color-blueprint-paper);
  }

  button[data-status="upcoming"] {
    background-color: var(--blueprint-line-faint);
    opacity: 0.7;
  }

  /* Step indicator - first div inside button */
  button > div:first-child {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    flex-shrink: 0;
    border: 1px solid var(--blueprint-line-light);
    background-color: var(--color-blueprint-paper);
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text);
    transition: all 0.15s;
  }

  button[data-status="completed"] > div:first-child {
    background-color: var(--success);
    border-color: var(--success);
    color: var(--color-blueprint-paper);
  }

  button[data-status="current"] > div:first-child {
    background-color: var(--color-blueprint-paper);
    border-color: var(--color-blueprint-paper);
    color: var(--primary);
  }

  button[data-status="upcoming"] > div:first-child {
    background-color: var(--blueprint-line-faint);
    border-color: var(--blueprint-line-light);
    color: var(--text-muted);
  }

  /* Checkmark SVG */
  svg {
    width: 1rem;
    height: 1rem;
  }

  /* Step number */
  button > div:first-child > span {
    line-height: 1;
  }

  /* Step content - second div inside button */
  button > div:last-child {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    flex: 1;
  }

  /* Step label - first div in content */
  button > div:last-child > div:first-child {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  button[data-status="current"] > div:last-child > div:first-child {
    color: var(--color-blueprint-paper);
  }

  /* Step description - second div in content */
  button > div:last-child > div:last-child {
    font-size: 0.75rem;
    color: var(--text-muted);
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

  /* Step connector */
  div[data-connector] {
    height: 1px;
    flex: 1;
    background-color: var(--blueprint-line-light);
    margin: 0 0.5rem;
    transition: background-color 0.3s;
    min-width: 1rem;
  }

  div[data-connector][data-completed] {
    background-color: var(--success);
  }

  @media (max-width: 768px) {
    div[role="navigation"] {
      flex-direction: column;
      gap: 0.5rem;
    }

    div[role="navigation"] > div {
      flex-direction: column;
      align-items: stretch;
    }

    div[data-connector] {
      width: 1px;
      height: 1rem;
      margin: 0;
      align-self: flex-start;
      margin-left: 1rem;
    }

    button > div:last-child > div {
      white-space: normal;
    }
  }
</style>
