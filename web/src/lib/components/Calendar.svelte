<script lang="ts">
  interface Props {
    value?: Date;
    min?: Date;
    max?: Date;
    disabled?: boolean;
    onSelect?: (date: Date) => void;
  }

  let {
    value = $bindable(new Date()),
    min,
    max,
    disabled = false,
    onSelect
  }: Props = $props();

  let currentMonth = $state(value ? new Date(value) : new Date());
  let selectedDate = $state(value);

  const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

  const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  function getDaysInMonth(date: Date): Date[] {
    const year = date.getFullYear();
    const month = date.getMonth();
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    const daysInMonth = lastDay.getDate();
    const startingDayOfWeek = firstDay.getDay();

    const days: Date[] = [];

    // Add empty slots for days before the 1st
    for (let i = 0; i < startingDayOfWeek; i++) {
      const prevMonthDay = new Date(year, month, -startingDayOfWeek + i + 1);
      days.push(prevMonthDay);
    }

    // Add days of current month
    for (let i = 1; i <= daysInMonth; i++) {
      days.push(new Date(year, month, i));
    }

    // Fill remaining slots to complete the week
    const remainingDays = 7 - (days.length % 7);
    if (remainingDays < 7) {
      for (let i = 1; i <= remainingDays; i++) {
        days.push(new Date(year, month + 1, i));
      }
    }

    return days;
  }

  function isSameDay(date1: Date, date2: Date | undefined): boolean {
    if (!date2) return false;
    return date1.getDate() === date2.getDate() &&
           date1.getMonth() === date2.getMonth() &&
           date1.getFullYear() === date2.getFullYear();
  }

  function isToday(date: Date): boolean {
    return isSameDay(date, new Date());
  }

  function isCurrentMonth(date: Date): boolean {
    return date.getMonth() === currentMonth.getMonth() &&
           date.getFullYear() === currentMonth.getFullYear();
  }

  function isDisabled(date: Date): boolean {
    if (disabled) return true;
    if (min && date < min) return true;
    if (max && date > max) return true;
    return false;
  }

  function selectDate(date: Date) {
    if (isDisabled(date)) return;
    selectedDate = date;
    value = date;
    if (onSelect) {
      onSelect(date);
    }
  }

  function previousMonth() {
    currentMonth = new Date(currentMonth.getFullYear(), currentMonth.getMonth() - 1);
  }

  function nextMonth() {
    currentMonth = new Date(currentMonth.getFullYear(), currentMonth.getMonth() + 1);
  }

  let days = $derived(getDaysInMonth(currentMonth));
</script>

<div class="calendar">
  <div class="calendar-header">
    <button
      type="button"
      class="calendar-nav-btn"
      onclick={previousMonth}
      disabled={disabled}
      aria-label="Previous month"
    >
      ←
    </button>
    <div class="calendar-month">
      {monthNames[currentMonth.getMonth()]} {currentMonth.getFullYear()}
    </div>
    <button
      type="button"
      class="calendar-nav-btn"
      onclick={nextMonth}
      disabled={disabled}
      aria-label="Next month"
    >
      →
    </button>
  </div>

  <div class="calendar-weekdays">
    {#each dayNames as dayName}
      <div class="calendar-weekday">{dayName}</div>
    {/each}
  </div>

  <div class="calendar-days">
    {#each days as day}
      <button
        type="button"
        class="calendar-day"
        class:calendar-day-today={isToday(day)}
        class:calendar-day-selected={isSameDay(day, selectedDate)}
        class:calendar-day-outside={!isCurrentMonth(day)}
        class:calendar-day-disabled={isDisabled(day)}
        onclick={() => selectDate(day)}
        disabled={isDisabled(day)}
      >
        {day.getDate()}
      </button>
    {/each}
  </div>
</div>

<style>
  .calendar {
    border: 1px solid var(--border-color-strong);
    background: var(--color-blueprint-paper);
    padding: 0.75rem;
    width: fit-content;
  }

  .calendar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    padding-top: 0.25rem;
  }

  .calendar-month {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text);
  }

  .calendar-nav-btn {
    width: 1.75rem;
    height: 1.75rem;
    border: 1px solid var(--border-color-strong);
    background: var(--color-blueprint-paper);
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 0.15s ease, background-color 0.15s ease;
    color: var(--text-muted);
  }

  .calendar-nav-btn:hover:not(:disabled) {
    border-color: var(--primary);
    color: var(--primary);
  }

  .calendar-nav-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 0.25rem;
    margin-bottom: 0.5rem;
  }

  .calendar-weekday {
    text-align: center;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-muted);
    width: 2.25rem;
  }

  .calendar-days {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 0.25rem;
  }

  .calendar-day {
    width: 2.25rem;
    height: 2.25rem;
    border: 1px solid transparent;
    background: var(--color-blueprint-paper);
    cursor: pointer;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 0.15s ease, background-color 0.15s ease;
    color: var(--text);
  }

  .calendar-day:hover:not(:disabled):not(.calendar-day-selected) {
    border-color: var(--primary);
    background-color: rgb(59 130 246 / 0.05);
  }

  .calendar-day-today {
    background: var(--color-gray-light);
    font-weight: 600;
  }

  .calendar-day-selected {
    background: var(--primary);
    color: var(--color-blueprint-paper);
    font-weight: 600;
  }

  .calendar-day-outside {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .calendar-day-disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
