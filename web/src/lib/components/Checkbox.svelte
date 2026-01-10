<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    indeterminate?: boolean;
    onchange?: (checked: boolean) => void;
  }

  let { checked = false, disabled = false, label = '', indeterminate = false, onchange }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    if (onchange) {
      onchange(target.checked);
    }
  }
</script>

<label>
  <input
    type="checkbox"
    bind:checked
    {disabled}
    {indeterminate}
    onchange={handleChange}
  />
  <span class="checkbox-indicator">
    {#if indeterminate}
      <span class="indeterminate-mark"></span>
    {:else if checked}
      <svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M2 8L6 12L14 4" stroke="currentColor" stroke-width="3" stroke-linecap="square" stroke-linejoin="miter"/>
      </svg>
    {/if}
  </span>
  {#if label}
    <span class="label-text">{label}</span>
  {/if}
</label>

<style>
  label {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    user-select: none;
  }

  label:has(input:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .checkbox-indicator {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    background-color: var(--color-blueprint-paper);
    border: 1px solid var(--color-border-strong);
    transition: background-color var(--transition-fast), border-color var(--transition-fast);
  }

  input:checked + .checkbox-indicator {
    background-color: var(--color-primary);
    border-color: var(--color-primary);
  }

  input:indeterminate + .checkbox-indicator {
    background-color: var(--color-blueprint-line-light);
    border-color: var(--color-blueprint-line-light);
  }

  input:focus + .checkbox-indicator {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  label:hover:not(:has(input:disabled)) .checkbox-indicator {
    border-color: var(--color-primary);
    border-width: 2px;
  }

  label:active:not(:has(input:disabled)) .checkbox-indicator {
    opacity: 0.9;
  }

  input:disabled + .checkbox-indicator {
    border-color: var(--color-gray-medium);
  }

  .indeterminate-mark {
    width: 0.625rem;
    height: 0.125rem;
    background-color: white;
  }

  svg {
    width: 0.875rem;
    height: 0.875rem;
    color: white;
  }

  .label-text {
    font-weight: 500;
  }
</style>
