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

<label class:disabled>
  <input
    type="checkbox"
    bind:checked
    {disabled}
    onchange={handleChange}
  />
  <span>
    <span></span>
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

  /* Switch track - first span after input */
  input + span {
    position: relative;
    display: inline-block;
    width: 48px;
    height: 24px;
    background-color: var(--gray-light);
    border: 1px solid var(--blueprint-line-light);
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  /* Switch thumb - nested span inside track */
  input + span > span {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    background-color: var(--white);
    border: 1px solid var(--blueprint-line-light);
    transition: transform 0.2s ease;
  }

  input:checked + span {
    background-color: var(--primary);
    border-color: var(--primary);
  }

  input:checked + span > span {
    transform: translateX(24px);
    background-color: var(--white);
    border-color: var(--white);
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
