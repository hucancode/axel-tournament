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
    value = 50,
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
    font-weight: 700;
    font-size: 0.875rem;
    color: var(--black);
  }

  .slider-value {
    background: var(--primary);
    border: 3px solid var(--black);
    border-radius: 4px;
    padding: 0.25rem 0.75rem;
    box-shadow: 2px 2px 0 0 var(--black);
    font-weight: 700;
    min-width: 3rem;
    text-align: center;
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    background: transparent;
    cursor: pointer;
    transition: none;
  }

  .slider::-webkit-slider-runnable-track {
    width: 100%;
    height: 1.5rem;
    background: linear-gradient(
      to right,
      var(--primary) 0%,
      var(--primary) var(--percentage),
      var(--white) var(--percentage),
      var(--white) 100%
    );
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 2rem;
    height: 2rem;
    background: var(--primary);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 2px 2px 0 0 var(--black);
    cursor: grab;
    transition: none;
    margin-top: -0.25rem;
  }

  .slider::-moz-range-track {
    width: 100%;
    height: 1.5rem;
    background: var(--white);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
  }

  .slider::-moz-range-progress {
    height: 1.5rem;
    background: var(--primary);
    border: none;
    border-radius: 4px 0 0 4px;
  }

  .slider::-moz-range-thumb {
    width: 2rem;
    height: 2rem;
    background: var(--primary);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 2px 2px 0 0 var(--black);
    cursor: grab;
    transition: none;
  }

  .slider::-webkit-slider-thumb:hover {
    background: var(--secondary);
  }

  .slider::-moz-range-thumb:hover {
    background: var(--secondary);
  }

  .slider::-webkit-slider-thumb:active {
    cursor: grabbing;
    box-shadow: none;
    transform: translate(1px, 1px);
  }

  .slider::-moz-range-thumb:active {
    cursor: grabbing;
    box-shadow: none;
    transform: translate(1px, 1px);
  }

  .slider:focus {
    outline: 3px solid var(--primary);
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
    box-shadow: 2px 2px 0 0 var(--gray-medium);
  }

  .slider:disabled::-moz-range-thumb {
    cursor: not-allowed;
    background: var(--gray-medium);
    border-color: var(--gray-medium);
    box-shadow: 2px 2px 0 0 var(--gray-medium);
  }
</style>
