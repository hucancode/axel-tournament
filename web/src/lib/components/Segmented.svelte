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

<div role="radiogroup" data-disabled={disabled}>
  {#each options as option (option.value)}
    <label>
      <input
        type="radio"
        {name}
        value={option.value}
        checked={option.value == value}
        onchange={handleChange}
        {disabled}
      />
      <span>{option.label}</span>
    </label>
  {/each}
</div>

<style>
  div[role="radiogroup"] {
    display: inline-flex;
    background-color: var(--color-blueprint-paper);
    border: 1px solid var(--color-border-strong);
    overflow: hidden;
  }

  div[data-disabled="true"] {
    opacity: 0.5;
    cursor: not-allowed;
  }

  label {
    position: relative;
    display: inline-flex;
    align-items: center;
    padding: 0.75rem 1.5rem;
    background-color: transparent;
    border-right: 1px solid var(--color-blueprint-line-faint);
    font-weight: 500;
    cursor: pointer;
    transition: background-color var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
    user-select: none;
  }

  label:last-child {
    border-right: 0;
  }

  label:has(input:disabled) {
    cursor: not-allowed;
  }

  input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  label:has(input:checked) {
    background-color: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }

  label:has(input:focus) {
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
    z-index: 10;
  }

  label:hover:has(input:not(:disabled):not(:checked)) {
    background-color: var(--color-blueprint-line-faint);
  }

  label:active:has(input:not(:disabled)) {
    background-color: var(--color-primary);
    color: white;
  }

  div[data-disabled="true"] label {
    border-color: var(--color-gray-medium);
  }
</style>
