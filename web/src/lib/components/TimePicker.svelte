<script lang="ts">
  interface Props {
    label?: string;
    value?: string;
    min?: string;
    max?: string;
    step?: number;
    disabled?: boolean;
    error?: string;
  }

  let {
    label = 'Time',
    value = $bindable(''),
    min,
    max,
    step,
    disabled = false,
    error = ''
  }: Props = $props();

  let showDropdown = $state(false);
  let timePickerRef: HTMLDivElement;
  let hours = $state('12');
  let minutes = $state('00');
  let ampm = $state<'AM' | 'PM'>('PM');

  $effect(() => {
    if (value) {
      const [h, m] = value.split(':');
      if (h && m) {
        const hour = parseInt(h);
        hours = String(hour === 0 ? 12 : hour > 12 ? hour - 12 : hour).padStart(2, '0');
        minutes = m;
        ampm = hour >= 12 ? 'PM' : 'AM';
      }
    }
  });

  const inputId = `time-picker-${Math.random().toString(36).slice(2)}`;

  const displayValue = $derived(() => {
    if (hours && minutes) {
      return `${hours}:${minutes} ${ampm}`;
    }
    return '';
  });

  function toggleDropdown() {
    if (!disabled) {
      showDropdown = !showDropdown;
    }
  }

  function updateValue() {
    const h = parseInt(hours);
    let hour24 = h;

    if (ampm === 'PM' && h !== 12) {
      hour24 = h + 12;
    } else if (ampm === 'AM' && h === 12) {
      hour24 = 0;
    }

    value = `${String(hour24).padStart(2, '0')}:${minutes}`;
  }

  function incrementHours() {
    let h = parseInt(hours);
    h = h === 12 ? 1 : h + 1;
    hours = String(h).padStart(2, '0');
    updateValue();
  }

  function decrementHours() {
    let h = parseInt(hours);
    h = h === 1 ? 12 : h - 1;
    hours = String(h).padStart(2, '0');
    updateValue();
  }

  function incrementMinutes() {
    let m = parseInt(minutes);
    m = (m + 1) % 60;
    minutes = String(m).padStart(2, '0');
    updateValue();
  }

  function decrementMinutes() {
    let m = parseInt(minutes);
    m = m === 0 ? 59 : m - 1;
    minutes = String(m).padStart(2, '0');
    updateValue();
  }

  function toggleAmPm() {
    ampm = ampm === 'AM' ? 'PM' : 'AM';
    updateValue();
  }

  function handleClickOutside(event: MouseEvent) {
    if (timePickerRef && !timePickerRef.contains(event.target as Node)) {
      showDropdown = false;
    }
  }

  $effect(() => {
    if (typeof window !== 'undefined') {
      if (showDropdown) {
        document.addEventListener('click', handleClickOutside);
      } else {
        document.removeEventListener('click', handleClickOutside);
      }

      return () => {
        document.removeEventListener('click', handleClickOutside);
      };
    }
  });
</script>

<div bind:this={timePickerRef}>
  <label for={inputId}>{label}</label>

  <div class="wrapper">
    <button
      type="button"
      class="trigger"
      onclick={toggleDropdown}
      {disabled}
    >
      <span class:placeholder={!displayValue()}>
        {displayValue() || 'Select time...'}
      </span>
    </button>

    {#if showDropdown}
      <div class="dropdown">
        <div class="picker">
          <div class="column">
            <button type="button" onclick={incrementHours} {disabled}>▲</button>
            <div class="display">{hours}</div>
            <button type="button" onclick={decrementHours} {disabled}>▼</button>
          </div>

          <div class="separator">:</div>

          <div class="column">
            <button type="button" onclick={incrementMinutes} {disabled}>▲</button>
            <div class="display">{minutes}</div>
            <button type="button" onclick={decrementMinutes} {disabled}>▼</button>
          </div>

          <div class="column">
            <button type="button" class="ampm" onclick={toggleAmPm} {disabled}>
              {ampm}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}
</div>

<style>
  .wrapper {
    position: relative;
  }

  .trigger {
    width: 100%;
    display: flex;
    align-items: center;
    padding: var(--spacing-2) var(--spacing-3);
    border: 1px solid var(--color-border);
    background: var(--color-bg-light);
    font-size: 1rem;
    cursor: pointer;
    transition: border-color var(--transition-fast);
    text-align: left;
    color: var(--color-fg);
  }

  .trigger:hover:not(:disabled) {
    border-color: var(--color-fg-dim);
  }

  .trigger:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .placeholder {
    color: var(--color-fg-dim);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + var(--spacing-2));
    left: 0;
    z-index: var(--z-dropdown);
    background: var(--color-bg-light);
    border: 1px solid var(--color-border);
    padding: var(--spacing-3);
  }

  .picker {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-2);
  }

  .column button {
    width: 2.5rem;
    height: 2rem;
    padding: 0;
    border: 1px solid var(--color-border);
    background: var(--color-bg-light);
    cursor: pointer;
    font-size: 0.875rem;
    transition: border-color var(--transition-fast);
    color: var(--color-fg-dim);
  }

  .column button:hover:not(:disabled) {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .display {
    width: 2.5rem;
    padding: var(--spacing-2);
    border: 1px solid var(--color-border);
    background: var(--color-bg-popup);
    text-align: center;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-fg);
  }

  .separator {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--color-fg-dim);
  }

  .ampm {
    width: 3rem;
    height: 100%;
    border: 1px solid var(--color-border);
    background: var(--color-bg-popup);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 600;
    transition: border-color var(--transition-fast);
  }

  .ampm:hover:not(:disabled) {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .error {
    margin-top: var(--spacing-2);
    color: var(--color-error);
    font-size: 0.875rem;
  }
</style>
