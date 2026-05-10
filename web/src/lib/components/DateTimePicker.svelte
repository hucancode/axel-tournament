<script lang="ts">
  import Calendar from './Calendar.svelte';

  interface Props {
    label?: string;
    value?: string; // ISO string or datetime-local format
    min?: string;
    max?: string;
    disabled?: boolean;
    error?: string;
  }

  let {
    label = 'Date & Time',
    value = $bindable(''),
    min,
    max,
    disabled = false,
    error = ''
  }: Props = $props();

  let showCalendar = $state(false);
  let dateTimePickerRef: HTMLDivElement;
  let selectedDate = $state<Date | undefined>(undefined);
  let selectedTime = $state('12:00');

  // Parse initial value
  $effect(() => {
    if (value) {
      const date = new Date(value);
      if (!isNaN(date.getTime())) {
        selectedDate = date;
        selectedTime = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
      }
    }
  });

  const inputId = `datetime-picker-${Math.random().toString(36).slice(2)}`;

  const displayValue = $derived(() => {
    if (selectedDate) {
      const year = selectedDate.getFullYear();
      const month = String(selectedDate.getMonth() + 1).padStart(2, '0');
      const day = String(selectedDate.getDate()).padStart(2, '0');
      return `${month}/${day}/${year} at ${selectedTime}`;
    }
    return '';
  });

  function toggleCalendar() {
    if (!disabled) {
      showCalendar = !showCalendar;
    }
  }

  function handleDateSelect(date: Date) {
    selectedDate = date;
    updateValue();
  }

  function handleTimeChange(e: Event) {
    const target = e.target as HTMLInputElement;
    selectedTime = target.value;
    updateValue();
  }

  function updateValue() {
    if (selectedDate && selectedTime) {
      const [hours, minutes] = selectedTime.split(':');
      const date = new Date(selectedDate);
      date.setHours(parseInt(hours), parseInt(minutes));

      // Format as datetime-local string (YYYY-MM-DDTHH:mm)
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      const hour = String(date.getHours()).padStart(2, '0');
      const minute = String(date.getMinutes()).padStart(2, '0');

      value = `${year}-${month}-${day}T${hour}:${minute}`;
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dateTimePickerRef && !dateTimePickerRef.contains(event.target as Node)) {
      showCalendar = false;
    }
  }

  $effect(() => {
    if (typeof window !== 'undefined') {
      if (showCalendar) {
        document.addEventListener('click', handleClickOutside);
      } else {
        document.removeEventListener('click', handleClickOutside);
      }

      return () => {
        document.removeEventListener('click', handleClickOutside);
      };
    }
  });

  const minDate = $derived(min ? new Date(min) : undefined);
  const maxDate = $derived(max ? new Date(max) : undefined);
</script>

<div class="datetime-picker-container" bind:this={dateTimePickerRef}>
  <label for={inputId} class="datetime-label">{label}</label>

  <div class="datetime-input-wrapper">
    <button
      type="button"
      class="datetime-input"
      class:datetime-input-disabled={disabled}
      onclick={toggleCalendar}
      {disabled}
    >
      <span class="datetime-value" class:datetime-placeholder={!displayValue()}>
        {displayValue() || 'Select date and time...'}
      </span>
      <svg class="icon datetime-icon"><use href="/icons.svg#i-calendar" /></svg>
    </button>

    {#if showCalendar}
      <div class="datetime-dropdown">
        <Calendar
          value={selectedDate}
          min={minDate}
          max={maxDate}
          {disabled}
          onSelect={handleDateSelect}
        />

        <div class="time-picker-section">
          <label for="time-input" class="time-label">Time:</label>
          <input
            id="time-input"
            type="time"
            class="time-input"
            bind:value={selectedTime}
            oninput={handleTimeChange}
            {disabled}
          />
        </div>
      </div>
    {/if}
  </div>

  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>

<style>
  .datetime-picker-container {
    width: 100%;
    position: relative;
  }

  .datetime-label {
    display: block;
    margin-bottom: var(--spacing-1);
    font-weight: 500;
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
  }

  .datetime-input-wrapper {
    position: relative;
  }

  .datetime-input {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-2) var(--spacing-3);
    border: var(--border-thin) solid var(--color-border);
    background: var(--color-bg-light);
    color: var(--color-fg);
    font-size: var(--font-size-md);
    cursor: pointer;
    transition: border-color var(--transition-fast);
    text-align: left;
  }

  .datetime-input:hover:not(:disabled) {
    border-color: var(--color-fg-dim);
  }

  .datetime-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .datetime-input-disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .datetime-value {
    flex: 1;
  }

  .datetime-placeholder {
    color: var(--color-fg-dim);
  }

  .datetime-icon {
    margin-left: var(--spacing-2);
    color: var(--color-fg-muted);
  }

  .datetime-dropdown {
    position: absolute;
    top: calc(100% + var(--spacing-2));
    left: 0;
    z-index: var(--z-dropdown);
    background: var(--color-bg-light);
    border: var(--border-thin) solid var(--color-border);
    box-shadow: var(--shadow-popup);
    padding: var(--spacing-3);
  }

  .time-picker-section {
    margin-top: var(--spacing-3);
    padding-top: var(--spacing-3);
    border-top: var(--border-thin) solid var(--color-border-light);
  }

  .time-label {
    display: block;
    margin-bottom: var(--spacing-1);
    font-weight: 500;
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
  }

  .time-input {
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    border: var(--border-thin) solid var(--color-border);
    background: var(--color-bg-light);
    color: var(--color-fg);
    font-size: var(--font-size-md);
    transition: border-color var(--transition-fast);
  }

  .time-input:hover:not(:disabled) {
    border-color: var(--color-fg-dim);
  }

  .time-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .form-error {
    margin-top: var(--spacing-2);
    color: var(--color-error);
    font-size: var(--font-size-sm);
  }
</style>
