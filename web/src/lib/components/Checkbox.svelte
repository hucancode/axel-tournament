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
    opacity: 0.6;
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
    width: 24px;
    height: 24px;
    background-color: var(--white);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
    transition: all 0.1s;
  }

  .checkbox-check {
    width: 16px;
    height: 16px;
    color: var(--black);
  }

  .checkbox-indeterminate {
    width: 12px;
    height: 3px;
    background-color: var(--black);
  }

  .checkbox-input:checked + .checkbox-box {
    background-color: var(--success);
  }

  .checkbox-input:indeterminate + .checkbox-box {
    background-color: var(--gray-light);
  }

  .checkbox-input:focus + .checkbox-box {
    outline: 3px solid var(--primary);
    outline-offset: 2px;
  }

  .checkbox-wrapper:hover .checkbox-box {
    box-shadow: 4px 4px 0 0 var(--black);
  }

  .checkbox-wrapper:active .checkbox-box {
    transform: translate(1px, 1px);
    box-shadow: 2px 2px 0 0 var(--black);
  }

  .checkbox-wrapper.disabled .checkbox-box {
    border-color: var(--gray-medium);
    box-shadow: 2px 2px 0 0 var(--gray-medium);
  }

  .checkbox-label {
    font-weight: 500;
    color: var(--black);
  }
</style>
