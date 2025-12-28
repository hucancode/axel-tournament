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

<div class="steps" role="navigation" aria-label="Progress steps">
  {#each steps as step, index (index)}
    <div class="step-wrapper">
      <button
        type="button"
        class="step"
        class:completed={getStepStatus(index) === 'completed'}
        class:current={getStepStatus(index) === 'current'}
        class:upcoming={getStepStatus(index) === 'upcoming'}
        class:clickable={clickable}
        disabled={!clickable}
        onclick={() => handleStepClick(index)}
        aria-current={getStepStatus(index) === 'current' ? 'step' : undefined}
      >
        <div class="step-indicator">
          {#if getStepStatus(index) === 'completed'}
            <svg
              class="checkmark"
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
            <span class="step-number">{index + 1}</span>
          {/if}
        </div>
        <div class="step-content">
          <div class="step-label">{step.label}</div>
          {#if step.description}
            <div class="step-description">{step.description}</div>
          {/if}
        </div>
      </button>
      {#if index < steps.length - 1}
        <div
          class="step-connector"
          class:completed={index < current}
        ></div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .steps {
    display: flex;
    width: 100%;
    gap: 0;
  }

  .step-wrapper {
    display: flex;
    flex: 1;
    align-items: center;
    min-width: 0;
  }

  .step {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background-color: var(--white);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
    cursor: default;
    transition: all 0.15s;
    flex: 1;
    min-width: 0;
  }

  .step.clickable {
    cursor: pointer;
  }

  .step.clickable:hover {
    transform: translate(1px, 1px);
    box-shadow: 2px 2px 0 0 var(--black);
  }

  .step.clickable:active {
    transform: translate(2px, 2px);
    box-shadow: 1px 1px 0 0 var(--black);
  }

  .step:focus {
    outline: 3px solid var(--primary);
    outline-offset: 2px;
    z-index: 1;
  }

  .step.current {
    background-color: var(--primary);
    border-width: 3px;
  }

  .step.upcoming {
    background-color: var(--gray-light);
    opacity: 0.7;
  }

  .step-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    border: 3px solid var(--black);
    border-radius: 50%;
    background-color: var(--white);
    font-weight: 700;
    font-size: 1rem;
    transition: all 0.15s;
  }

  .step.completed .step-indicator {
    background-color: var(--success);
    color: var(--black);
  }

  .step.current .step-indicator {
    background-color: var(--black);
    color: var(--white);
    box-shadow: 2px 2px 0 0 rgba(0, 0, 0, 0.3);
  }

  .step.upcoming .step-indicator {
    background-color: var(--gray-light);
    color: var(--gray-medium);
  }

  .checkmark {
    width: 1.25rem;
    height: 1.25rem;
  }

  .step-number {
    line-height: 1;
  }

  .step-content {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    flex: 1;
  }

  .step-label {
    font-weight: 700;
    font-size: 0.875rem;
    color: var(--black);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .step-description {
    font-size: 0.75rem;
    color: var(--gray-dark);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .step.upcoming .step-label,
  .step.upcoming .step-description {
    opacity: 0.6;
  }

  .step-connector {
    height: 3px;
    flex: 1;
    background-color: var(--gray-medium);
    margin: 0 0.5rem;
    transition: background-color 0.3s;
    min-width: 1rem;
  }

  .step-connector.completed {
    background-color: var(--success);
  }

  @media (max-width: 768px) {
    .steps {
      flex-direction: column;
      gap: 0.5rem;
    }

    .step-wrapper {
      flex-direction: column;
      align-items: stretch;
    }

    .step-connector {
      width: 3px;
      height: 1rem;
      margin: 0;
      align-self: flex-start;
      margin-left: 1.25rem;
    }

    .step-label {
      white-space: normal;
    }

    .step-description {
      white-space: normal;
    }
  }
</style>
