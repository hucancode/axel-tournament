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

<div>
  {#if label}
    <label for={sliderId}>
      {label}
      {#if showValue}
        <span>{value}</span>
      {/if}
    </label>
  {/if}
  <input
    id={sliderId}
    type="range"
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
  div {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
  }

  label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text);
  }

  span {
    background: var(--primary);
    color: var(--white);
    border: 1px solid var(--blueprint-line);
    padding: 0.25rem 0.75rem;
    font-weight: 600;
    min-width: 3rem;
    text-align: center;
  }

  input {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    background: transparent;
    cursor: pointer;
  }

  input::-webkit-slider-runnable-track {
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
  }

  input::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 1.25rem;
    height: 1.25rem;
    background: var(--primary);
    border: 1px solid var(--blueprint-line);
    cursor: grab;
    margin-top: -0.125rem;
  }

  input::-moz-range-track {
    width: 100%;
    height: 1rem;
    background: var(--white);
    border: 1px solid var(--blueprint-line-light);
  }

  input::-moz-range-progress {
    height: 1rem;
    background: var(--primary);
    border: none;
  }

  input::-moz-range-thumb {
    width: 1.25rem;
    height: 1.25rem;
    background: var(--primary);
    border: 1px solid var(--blueprint-line);
    cursor: grab;
  }

  input::-webkit-slider-thumb:hover {
    border-width: 2px;
    border-color: var(--blueprint-line);
  }

  input::-moz-range-thumb:hover {
    border-width: 2px;
    border-color: var(--blueprint-line);
  }

  input::-webkit-slider-thumb:active {
    cursor: grabbing;
    opacity: 0.9;
  }

  input::-moz-range-thumb:active {
    cursor: grabbing;
    opacity: 0.9;
  }

  input:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input:disabled::-webkit-slider-runnable-track {
    background: linear-gradient(
      to right,
      var(--gray-medium) 0%,
      var(--gray-medium) var(--percentage),
      var(--white) var(--percentage),
      var(--white) 100%
    );
  }

  input:disabled::-moz-range-progress {
    background: var(--gray-medium);
  }

  input:disabled::-webkit-slider-thumb {
    cursor: not-allowed;
    background: var(--gray-medium);
    border-color: var(--gray-medium);
  }

  input:disabled::-moz-range-thumb {
    cursor: not-allowed;
    background: var(--gray-medium);
    border-color: var(--gray-medium);
  }
</style>
