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

<label class:disabled>
  <input
    type="checkbox"
    bind:checked
    {disabled}
    {indeterminate}
    onchange={handleChange}
  />
  <span>
    {#if indeterminate}
      <span></span>
    {:else if checked}
      <svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M2 8L6 12L14 4" stroke="currentColor" stroke-width="3" stroke-linecap="square" stroke-linejoin="miter"/>
      </svg>
    {/if}
  </span>
  {#if label}
    <span>{label}</span>
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

  label.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  /* Checkbox box - first span after input */
  input + span {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background-color: var(--white);
    border: 1px solid var(--blueprint-line-light);
    transition: border-color 0.15s ease;
  }

  /* Checkmark SVG */
  input + span svg {
    width: 14px;
    height: 14px;
    color: var(--white);
  }

  /* Indeterminate indicator - nested span */
  input + span > span {
    width: 10px;
    height: 2px;
    background-color: var(--white);
  }

  input:checked + span {
    background-color: var(--primary);
    border-color: var(--primary);
  }

  input:indeterminate + span {
    background-color: var(--blueprint-line-light);
    border-color: var(--blueprint-line-light);
  }

  input:focus + span {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  label:hover input + span {
    border-color: var(--primary);
    border-width: 2px;
  }

  label:active input + span {
    opacity: 0.9;
  }

  label.disabled input + span {
    border-color: var(--gray-medium);
  }

  /* Label text - second span (if exists) */
  input + span + span {
    font-weight: 500;
    color: var(--text);
  }
</style>
