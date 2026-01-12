<script lang="ts">
  import Calendar from './Calendar.svelte';

  interface Props {
    label?: string;
    value?: string;
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

<div bind:this={datePickerRef}>
  <label for={inputId}>{label}</label>

  <div class="wrapper">
    <button type="button" class="trigger" onclick={toggleCalendar} {disabled}>
      <span class:placeholder={!displayValue()}>
        {displayValue() || 'Select a date...'}
      </span>
    </button>

    {#if showCalendar}
      <div class="dropdown">
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
  }

  .error {
    margin-top: var(--spacing-2);
    color: var(--color-error);
    font-size: 0.875rem;
  }
</style>
