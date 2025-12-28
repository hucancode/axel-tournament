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
    value = options[0]?.value,
    disabled = false,
    onchange
  }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
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
        value={option.value}
        checked={option.value == value}
        onchange={handleChange}
      />
      {option.label}
    </label>
  {/each}
</div>

<style>
  div[role="radiogroup"] {
    display: inline-flex;
    background-color: var(--gray-light);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
    overflow: hidden;
  }

  div[role="radiogroup"].disabled {
    opacity: 0.6;
    cursor: not-allowed;
    border-color: var(--gray-medium);
    box-shadow: 2px 2px 0 0 var(--gray-medium);
  }

  label {
    padding: 0.75rem 1.5rem;
    background-color: transparent;
    border-right: 3px solid var(--black);
    font-weight: 500;
    color: var(--black);
    cursor: pointer;
    transition: all 0.1s;
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
    background-color: var(--white);
  }

  label:has(input:checked) {
    background-color: var(--primary);
  }

  label:has(input:focus) {
    outline: 3px solid var(--primary);
    outline-offset: -3px;
    z-index: 1;
  }

  label:active:not(:has(input:disabled)) {
    background-color: var(--primary);
  }

  label:has(input:disabled) {
    cursor: not-allowed;
  }

  div[role="radiogroup"].disabled label {
    border-color: var(--gray-medium);
  }
</style>
