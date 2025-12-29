<script lang="ts">
  interface Props {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    showValue?: boolean;
    disabled?: boolean;
    onValueChange?: (value: number) => void;
  }

  let {
    value = $bindable(50),
    min = 0,
    max = 100,
    step = 1,
    label = '',
    showValue = true,
    disabled = false,
    onValueChange = () => {}
  }: Props = $props();

  const sliderId = `slider-${Math.random().toString(36).slice(2)}`;
  let percentage = $derived(((value - min) / (max - min)) * 100);

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    const newValue = Number(target.value);
    value = newValue;
    onValueChange(newValue);
  }
</script>

<div class="slider-container">
  {#if label}
    <label for={sliderId} class="slider-label">
      {label}
      {#if showValue}
        <span class="slider-value">{value}</span>
      {/if}
    </label>
  {/if}
  <input
    id={sliderId}
    type="range"
    class="slider"
    style="--percentage: {percentage}%"
    {value}
    {min}
    {max}
    {step}
    {disabled}
    oninput={handleInput}
  />
</div>

<style>
  .slider-container {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
  }

  .slider-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text);
  }

  .slider-value {
    background: var(--primary);
    color: var(--white);
    border: 1px solid var(--blueprint-line);
    border-radius: 0;
    padding: 0.25rem 0.75rem;
    font-weight: 600;
    min-width: 3rem;
    text-align: center;
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    background: transparent;
    cursor: pointer;
  }

  .slider::-webkit-slider-runnable-track {
    width: 100%;
    height: 1rem;
    background: linear-gradient(
      to right,
      var(--primary) 0%,
      var(--primary) var(--percentage),
      var(--white) var(--percentage),
      var(--white) 100%
    );
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 1.25rem;
    height: 1.25rem;
    background: var(--primary);
    border: 1px solid var(--blueprint-line);
    border-radius: 0;
    cursor: grab;
    margin-top: -0.125rem;
  }

  .slider::-moz-range-track {
    width: 100%;
    height: 1rem;
    background: var(--white);
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
  }

  .slider::-moz-range-progress {
    height: 1rem;
    background: var(--primary);
    border: none;
  }

  .slider::-moz-range-thumb {
    width: 1.25rem;
    height: 1.25rem;
    background: var(--primary);
    border: 1px solid var(--blueprint-line);
    border-radius: 0;
    cursor: grab;
  }

  .slider::-webkit-slider-thumb:hover {
    border-width: 2px;
    border-color: var(--blueprint-line);
  }

  .slider::-moz-range-thumb:hover {
    border-width: 2px;
    border-color: var(--blueprint-line);
  }

  .slider::-webkit-slider-thumb:active {
    cursor: grabbing;
    opacity: 0.9;
  }

  .slider::-moz-range-thumb:active {
    cursor: grabbing;
    opacity: 0.9;
  }

  .slider:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  .slider:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .slider:disabled::-webkit-slider-runnable-track {
    background: linear-gradient(
      to right,
      var(--gray-medium) 0%,
      var(--gray-medium) var(--percentage),
      var(--white) var(--percentage),
      var(--white) 100%
    );
  }

  .slider:disabled::-moz-range-progress {
    background: var(--gray-medium);
  }

  .slider:disabled::-webkit-slider-thumb {
    cursor: not-allowed;
    background: var(--gray-medium);
    border-color: var(--gray-medium);
  }

  .slider:disabled::-moz-range-thumb {
    cursor: not-allowed;
    background: var(--gray-medium);
    border-color: var(--gray-medium);
  }
</style>
