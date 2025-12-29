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

<label class="switch-wrapper" class:disabled>
  <input
    type="checkbox"
    class="switch-input"
    bind:checked
    {disabled}
    onchange={handleChange}
  />
  <span class="switch-track">
    <span class="switch-thumb"></span>
  </span>
  {#if label}
    <span class="switch-label">{label}</span>
  {/if}
</label>

<style>
  .switch-wrapper {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    user-select: none;
  }

  .switch-wrapper.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .switch-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .switch-track {
    position: relative;
    display: inline-block;
    width: 48px;
    height: 24px;
    background-color: var(--gray-light);
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    background-color: var(--white);
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
    transition: transform 0.2s ease;
  }

  .switch-input:checked + .switch-track {
    background-color: var(--primary);
    border-color: var(--primary);
  }

  .switch-input:checked + .switch-track .switch-thumb {
    transform: translateX(24px);
    background-color: var(--white);
    border-color: var(--white);
  }

  .switch-input:focus + .switch-track {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  .switch-wrapper:hover .switch-track {
    border-color: var(--primary);
    border-width: 2px;
  }

  .switch-wrapper:active .switch-track {
    opacity: 0.9;
  }

  .switch-wrapper.disabled .switch-track {
    border-color: var(--gray-medium);
  }

  .switch-label {
    font-weight: 500;
    color: var(--text);
  }
</style>
