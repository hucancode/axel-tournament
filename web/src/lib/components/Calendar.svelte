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

    for (let i = 0; i < startingDayOfWeek; i++) {
      const prevMonthDay = new Date(year, month, -startingDayOfWeek + i + 1);
      days.push(prevMonthDay);
    }

    for (let i = 1; i <= daysInMonth; i++) {
      days.push(new Date(year, month, i));
    }

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
  <header>
    <button type="button" onclick={previousMonth} disabled={disabled} aria-label="Previous month">←</button>
    <span>{monthNames[currentMonth.getMonth()]} {currentMonth.getFullYear()}</span>
    <button type="button" onclick={nextMonth} disabled={disabled} aria-label="Next month">→</button>
  </header>

  <div class="weekdays">
    {#each dayNames as dayName}
      <div>{dayName}</div>
    {/each}
  </div>

  <div class="days">
    {#each days as day}
      <button
        type="button"
        data-today={isToday(day) || undefined}
        data-selected={isSameDay(day, selectedDate) || undefined}
        data-outside={!isCurrentMonth(day) || undefined}
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
    border: 1px solid var(--color-border);
    background: var(--color-bg-light);
    padding: var(--spacing-3);
    width: fit-content;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-3);
  }

  header span {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-fg);
  }

  header button {
    width: 1.75rem;
    height: 1.75rem;
    border: 1px solid var(--color-border);
    background: var(--color-bg-light);
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color var(--transition-fast);
    color: var(--color-fg-dim);
    padding: 0;
  }

  header button:hover:not(:disabled) {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  header button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 0.25rem;
    margin-bottom: var(--spacing-2);
  }

  .weekdays div {
    text-align: center;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-fg-dim);
    width: 2.25rem;
  }

  .days {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 0.25rem;
  }

  .days button {
    width: 2.25rem;
    height: 2.25rem;
    border: 1px solid transparent;
    background: var(--color-bg-light);
    cursor: pointer;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color var(--transition-fast);
    color: var(--color-fg);
    padding: 0;
  }

  .days button:hover:not(:disabled):not([data-selected]) {
    border-color: var(--color-primary);
  }

  .days button[data-today] {
    background: var(--color-bg-popup);
    font-weight: 600;
  }

  .days button[data-selected] {
    background: var(--color-primary);
    color: var(--color-bg-dark);
    font-weight: 600;
  }

  .days button[data-outside] {
    color: var(--color-fg-dim);
    opacity: 0.5;
  }

  .days button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
