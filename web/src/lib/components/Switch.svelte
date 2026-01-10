<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    onchange?: (checked: boolean) => void;
  }

  let { checked = false, disabled = false, label = '', onchange }: Props = $props();

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
    onchange={handleChange}
  />
  <span class="switch-track">
    <span class="switch-thumb"></span>
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

  .switch-track {
    position: relative;
    display: inline-block;
    width: 3rem;
    height: 1.5rem;
    background-color: var(--color-gray-light);
    border: 1px solid var(--color-border-strong);
    transition: background-color var(--transition-fast), border-color var(--transition-fast);
  }

  .switch-thumb {
    position: absolute;
    top: 0.125rem;
    left: 0.125rem;
    width: 1.125rem;
    height: 1.125rem;
    background-color: var(--color-blueprint-paper);
    border: 1px solid var(--color-border-strong);
    transition: transform var(--transition-fast), background-color var(--transition-fast), border-color var(--transition-fast);
  }

  input:checked + .switch-track {
    background-color: var(--color-primary);
    border-color: var(--color-primary);
  }

  input:checked + .switch-track .switch-thumb {
    transform: translateX(1.5rem);
    background-color: var(--color-blueprint-paper);
    border-color: white;
  }

  input:focus + .switch-track {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  label:hover:not(:has(input:disabled)) .switch-track {
    border-color: var(--color-primary);
    border-width: 2px;
  }

  label:hover:not(:has(input:disabled)) .switch-thumb {
    /* Adjust thumb position to account for increased border */
    top: 0.0625rem;
    left: 0.0625rem;
  }

  label:active:not(:has(input:disabled)) .switch-track {
    opacity: 0.9;
  }

  input:disabled + .switch-track {
    border-color: var(--color-gray-medium);
  }

  .label-text {
    font-weight: 500;
  }
</style>
