<script lang="ts">
  interface SegmentedOption {
    value: string;
    label: string;
  }

  interface Props {
    options: SegmentedOption[];
    value?: string;
    disabled?: boolean;
    name?: string;
    onchange?: (value: string) => void;
  }

  let {
    options,
    value = $bindable(options[0]?.value),
    disabled = false,
    name = `segmented-${Math.random().toString(36).slice(2)}`,
    onchange
  }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    if (onchange) {
      onchange(target.value);
    }
  }
</script>

<div class:disabled role="radiogroup">
  {#each options as option, index (option.value)}
    <label>
      <input
        type="radio"
        {name}
        value={option.value}
        checked={option.value == value}
        onchange={handleChange}
        {disabled}
      />
      {option.label}
    </label>
  {/each}
</div>

<style>
  div[role="radiogroup"] {
    display: inline-flex;
    background-color: var(--white);
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
    overflow: hidden;
  }

  div[role="radiogroup"].disabled {
    opacity: 0.5;
    cursor: not-allowed;
    border-color: var(--gray-medium);
  }

  label {
    padding: 0.75rem 1.5rem;
    background-color: transparent;
    border-right: 1px solid var(--blueprint-line-faint);
    font-weight: 500;
    color: var(--text);
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease;
    position: relative;
    display: inline-flex;
    align-items: center;
    user-select: none;
  }

  label:last-child {
    border-right: none;
  }

  input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  label:hover:not(:has(input:disabled)):not(:has(input:checked)) {
    background-color: var(--blueprint-line-faint);
  }

  label:has(input:checked) {
    background-color: var(--primary);
    color: var(--white);
    border-color: var(--primary);
  }

  label:has(input:focus) {
    outline: 2px solid var(--primary);
    outline-offset: -2px;
    z-index: 1;
  }

  label:active:not(:has(input:disabled)) {
    background-color: var(--primary);
    color: var(--white);
  }

  label:has(input:disabled) {
    cursor: not-allowed;
  }

  div[role="radiogroup"].disabled label {
    border-color: var(--gray-medium);
  }
</style>
