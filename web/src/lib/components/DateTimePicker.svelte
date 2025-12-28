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
      <span class="datetime-icon">📅</span>
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
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: var(--black);
    font-size: 0.875rem;
  }

  .datetime-input-wrapper {
    position: relative;
  }

  .datetime-input {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border: 3px solid var(--black);
    background: var(--white);
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
    transition: none;
    text-align: left;
    box-shadow: none;
  }

  .datetime-input:hover:not(:disabled) {
    box-shadow: 3px 3px 0 0 var(--primary);
  }

  .datetime-input:focus {
    outline: none;
    box-shadow: 3px 3px 0 0 var(--primary);
  }

  .datetime-input-disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .datetime-value {
    flex: 1;
  }

  .datetime-placeholder {
    color: var(--gray-medium);
  }

  .datetime-icon {
    margin-left: 0.5rem;
    font-size: 1rem;
  }

  .datetime-dropdown {
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    z-index: 50;
    background: var(--white);
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 6px 6px 0 0 var(--black);
    padding: 1rem;
  }

  .time-picker-section {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 3px solid var(--black);
  }

  .time-label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: var(--black);
    font-size: 0.875rem;
  }

  .time-input {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 3px solid var(--black);
    background: var(--white);
    border-radius: 4px;
    font-size: 1rem;
    transition: none;
    box-shadow: none;
  }

  .time-input:hover:not(:disabled) {
    box-shadow: 3px 3px 0 0 var(--primary);
  }

  .time-input:focus {
    outline: none;
    box-shadow: 3px 3px 0 0 var(--primary);
  }

  .form-error {
    margin-top: 0.5rem;
    color: var(--error);
    font-size: 0.875rem;
  }
</style>
