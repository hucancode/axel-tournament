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

<label class="checkbox-wrapper" class:disabled>
  <input
    type="checkbox"
    class="checkbox-input"
    bind:checked
    {disabled}
    {indeterminate}
    onchange={handleChange}
  />
  <span class="checkbox-box">
    {#if indeterminate}
      <span class="checkbox-indeterminate"></span>
    {:else if checked}
      <svg class="checkbox-check" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M2 8L6 12L14 4" stroke="currentColor" stroke-width="3" stroke-linecap="square" stroke-linejoin="miter"/>
      </svg>
    {/if}
  </span>
  {#if label}
    <span class="checkbox-label">{label}</span>
  {/if}
</label>

<style>
  .checkbox-wrapper {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    user-select: none;
  }

  .checkbox-wrapper.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .checkbox-box {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background-color: var(--white);
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
    transition: border-color 0.15s ease;
  }

  .checkbox-check {
    width: 14px;
    height: 14px;
    color: var(--white);
  }

  .checkbox-indeterminate {
    width: 10px;
    height: 2px;
    background-color: var(--white);
  }

  .checkbox-input:checked + .checkbox-box {
    background-color: var(--primary);
    border-color: var(--primary);
  }

  .checkbox-input:indeterminate + .checkbox-box {
    background-color: var(--blueprint-line-light);
    border-color: var(--blueprint-line-light);
  }

  .checkbox-input:focus + .checkbox-box {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  .checkbox-wrapper:hover .checkbox-box {
    border-color: var(--primary);
    border-width: 2px;
  }

  .checkbox-wrapper:active .checkbox-box {
    opacity: 0.9;
  }

  .checkbox-wrapper.disabled .checkbox-box {
    border-color: var(--gray-medium);
  }

  .checkbox-label {
    font-weight: 500;
    color: var(--text);
  }
</style>
