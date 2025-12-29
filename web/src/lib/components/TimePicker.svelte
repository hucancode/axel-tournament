<script lang="ts">
  interface Props {
    label?: string;
    value?: string; // HH:mm format
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

  // Parse initial value
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

<div class="time-picker-container" bind:this={timePickerRef}>
  <label for={inputId} class="time-label">{label}</label>

  <div class="time-input-wrapper">
    <button
      type="button"
      class="time-input"
      class:time-input-disabled={disabled}
      onclick={toggleDropdown}
      {disabled}
    >
      <span class="time-value" class:time-placeholder={!displayValue()}>
        {displayValue() || 'Select time...'}
      </span>
      <span class="time-icon">🕐</span>
    </button>

    {#if showDropdown}
      <div class="time-dropdown">
        <div class="time-picker">
          <div class="time-column">
            <button
              type="button"
              class="time-btn"
              onclick={incrementHours}
              {disabled}
            >
              ▲
            </button>
            <div class="time-display">{hours}</div>
            <button
              type="button"
              class="time-btn"
              onclick={decrementHours}
              {disabled}
            >
              ▼
            </button>
          </div>

          <div class="time-separator">:</div>

          <div class="time-column">
            <button
              type="button"
              class="time-btn"
              onclick={incrementMinutes}
              {disabled}
            >
              ▲
            </button>
            <div class="time-display">{minutes}</div>
            <button
              type="button"
              class="time-btn"
              onclick={decrementMinutes}
              {disabled}
            >
              ▼
            </button>
          </div>

          <div class="time-column">
            <button
              type="button"
              class="ampm-btn"
              onclick={toggleAmPm}
              {disabled}
            >
              {ampm}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>

  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>

<style>
  .time-picker-container {
    width: 100%;
    position: relative;
  }

  .time-label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: var(--text);
    font-size: 0.875rem;
  }

  .time-input-wrapper {
    position: relative;
  }

  .time-input {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border-color-strong);
    background: var(--white);
    font-size: 1rem;
    cursor: pointer;
    transition: border-color 0.15s ease;
    text-align: left;
    color: var(--text);
  }

  .time-input:hover:not(:disabled) {
    border-color: var(--primary);
  }

  .time-input:focus {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgb(59 130 246 / 0.1);
  }

  .time-input-disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .time-value {
    flex: 1;
  }

  .time-placeholder {
    color: var(--text-muted);
  }

  .time-icon {
    margin-left: 0.5rem;
    font-size: 1rem;
  }

  .time-dropdown {
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    z-index: 50;
    background: var(--white);
    border: 1px solid var(--border-color-strong);
    padding: 1rem;
  }

  .time-picker {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .time-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .time-btn {
    width: 2.5rem;
    height: 2rem;
    border: 1px solid var(--border-color-strong);
    background: var(--white);
    cursor: pointer;
    font-size: 0.875rem;
    transition: border-color 0.15s ease, background-color 0.15s ease;
    color: var(--text-muted);
  }

  .time-btn:hover:not(:disabled) {
    border-color: var(--primary);
    color: var(--primary);
  }

  .time-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .time-display {
    width: 2.5rem;
    padding: 0.5rem;
    border: 1px solid var(--border-color-strong);
    background: var(--white);
    text-align: center;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .time-separator {
    font-size: 1.5rem;
    font-weight: 600;
    padding: 0 0.25rem;
    color: var(--text-muted);
  }

  .ampm-btn {
    width: 3rem;
    height: 100%;
    border: 1px solid var(--border-color-strong);
    background: var(--gray-light);
    color: var(--text);
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 600;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .ampm-btn:hover:not(:disabled) {
    border-color: var(--primary);
    background: rgb(59 130 246 / 0.1);
    color: var(--primary);
  }

  .ampm-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-error {
    margin-top: 0.5rem;
    color: var(--error);
    font-size: 0.875rem;
  }
</style>
