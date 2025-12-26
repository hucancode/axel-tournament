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
    opacity: 0.6;
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
    width: 60px;
    height: 32px;
    background-color: var(--gray-light);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 3px 3px 0 0 var(--black);
    transition: all 0.1s;
  }

  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 22px;
    height: 22px;
    background-color: var(--white);
    border: 3px solid var(--black);
    border-radius: 2px;
    transition: all 0.2s;
  }

  .switch-input:checked + .switch-track {
    background-color: var(--success);
  }

  .switch-input:checked + .switch-track .switch-thumb {
    transform: translateX(28px);
    background-color: var(--white);
  }

  .switch-input:focus + .switch-track {
    outline: 3px solid var(--primary);
    outline-offset: 2px;
  }

  .switch-wrapper:hover .switch-track {
    box-shadow: 4px 4px 0 0 var(--black);
  }

  .switch-wrapper:active .switch-track {
    transform: translate(1px, 1px);
    box-shadow: 2px 2px 0 0 var(--black);
  }

  .switch-wrapper.disabled .switch-track {
    border-color: var(--gray-medium);
    box-shadow: 2px 2px 0 0 var(--gray-medium);
  }

  .switch-label {
    font-weight: 500;
    color: var(--black);
  }
</style>
