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
    background: var(--color-bg-light);
    border: 1px solid var(--color-border);
  }

  div[data-disabled="true"] {
    opacity: 0.5;
    cursor: not-allowed;
  }

  label {
    display: inline-flex;
    align-items: center;
    padding: var(--spacing-2) var(--spacing-4);
    background: transparent;
    border-right: 1px solid var(--color-border-light);
    font-weight: 500;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
    user-select: none;
    color: var(--color-fg-muted);
    margin: 0;
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
    background: var(--color-primary);
    color: var(--color-bg-dark);
  }

  label:has(input:focus) {
    outline: 1px solid var(--color-primary);
    outline-offset: -1px;
  }

  label:hover:has(input:not(:disabled):not(:checked)) {
    background: var(--color-bg-popup);
  }
</style>
