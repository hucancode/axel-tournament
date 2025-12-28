<script lang="ts">
  import Calendar from './Calendar.svelte';

  interface Props {
    label?: string;
    value?: string; // ISO string or date format (YYYY-MM-DD)
    min?: string;
    max?: string;
    disabled?: boolean;
    error?: string;
  }

  let {
    label = 'Date',
    value = $bindable(''),
    min,
    max,
    disabled = false,
    error = ''
  }: Props = $props();

  let showCalendar = $state(false);
  let datePickerRef: HTMLDivElement;
  let selectedDate = $state<Date | undefined>(undefined);

  // Parse initial value
  $effect(() => {
    if (value) {
      const date = new Date(value);
      if (!isNaN(date.getTime())) {
        selectedDate = date;
      }
    }
  });

  const inputId = `date-picker-${Math.random().toString(36).slice(2)}`;

  const displayValue = $derived(() => {
    if (selectedDate) {
      const year = selectedDate.getFullYear();
      const month = String(selectedDate.getMonth() + 1).padStart(2, '0');
      const day = String(selectedDate.getDate()).padStart(2, '0');
      return `${month}/${day}/${year}`;
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

    // Format as YYYY-MM-DD
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    value = `${year}-${month}-${day}`;

    showCalendar = false;
  }

  function handleClickOutside(event: MouseEvent) {
    if (datePickerRef && !datePickerRef.contains(event.target as Node)) {
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

<div class="date-picker-container" bind:this={datePickerRef}>
  <label for={inputId} class="date-label">{label}</label>

  <div class="date-input-wrapper">
    <button
      type="button"
      class="date-input"
      class:date-input-disabled={disabled}
      onclick={toggleCalendar}
      {disabled}
    >
      <span class="date-value" class:date-placeholder={!displayValue()}>
        {displayValue() || 'Select a date...'}
      </span>
      <span class="date-icon">📅</span>
    </button>

    {#if showCalendar}
      <div class="date-dropdown">
        <Calendar
          value={selectedDate}
          min={minDate}
          max={maxDate}
          {disabled}
          onSelect={handleDateSelect}
        />
      </div>
    {/if}
  </div>

  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>

<style>
  .date-picker-container {
    width: 100%;
    position: relative;
  }

  .date-label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: var(--black);
    font-size: 0.875rem;
  }

  .date-input-wrapper {
    position: relative;
  }

  .date-input {
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

  .date-input:hover:not(:disabled) {
    box-shadow: 3px 3px 0 0 var(--primary);
  }

  .date-input:focus {
    outline: none;
    box-shadow: 3px 3px 0 0 var(--primary);
  }

  .date-input-disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .date-value {
    flex: 1;
  }

  .date-placeholder {
    color: var(--gray-medium);
  }

  .date-icon {
    margin-left: 0.5rem;
    font-size: 1rem;
  }

  .date-dropdown {
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    z-index: 50;
  }

  .form-error {
    margin-top: 0.5rem;
    color: var(--error);
    font-size: 0.875rem;
  }
</style>
