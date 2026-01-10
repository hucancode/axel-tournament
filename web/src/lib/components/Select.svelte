<script lang="ts">
  interface SelectOption {
    value: string;
    label: string;
    disabled?: boolean;
  }

  interface Props {
    options: SelectOption[];
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    label?: string;
    error?: string;
    onchange?: () => void;
  }

  let {
    options,
    value = $bindable(''),
    placeholder = 'Select an option...',
    disabled = false,
    label,
    error = '',
    onchange
  }: Props = $props();

  let isOpen = $state(false);
  let selectRef: HTMLDivElement;

  const inputId = `select-${Math.random().toString(36).slice(2)}`;
  const dropdownId = `dropdown-${Math.random().toString(36).slice(2)}`;

  const selectedOption = $derived(
    options.find(opt => opt.value === value)
  );

  const displayText = $derived(
    selectedOption ? selectedOption.label : placeholder
  );

  function toggleDropdown() {
    if (!disabled) {
      isOpen = !isOpen;
    }
  }

  function selectOption(option: SelectOption) {
    if (!option.disabled) {
      value = option.value;
      isOpen = false;
      if (onchange) {
        onchange();
      }
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (selectRef && !selectRef.contains(event.target as Node)) {
      isOpen = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (disabled) return;

    if (event.key === 'Escape') {
      isOpen = false;
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleDropdown();
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (!isOpen) {
        isOpen = true;
      }
    }
  }

  $effect(() => {
    if (typeof window !== 'undefined') {
      if (isOpen) {
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

<div class="select-wrapper">
  {#if label}
    <label for={inputId}>{label}</label>
  {/if}

  <div
    class="select-container"
    bind:this={selectRef}
    role="combobox"
    aria-expanded={isOpen}
    aria-haspopup="listbox"
    aria-controls={dropdownId}
    tabindex={disabled ? -1 : 0}
    onkeydown={handleKeydown}
  >
    <button
      type="button"
      id={inputId}
      class="select-trigger"
      data-open={isOpen}
      data-placeholder={!selectedOption}
      onclick={toggleDropdown}
      {disabled}
    >
      <span class="select-value">{displayText}</span>
      <span class="select-arrow" data-open={isOpen}>▼</span>
    </button>

    {#if isOpen}
      <div id={dropdownId} class="select-dropdown" role="listbox">
        {#each options as option}
          <button
            type="button"
            class="select-option"
            data-selected={option.value === value}
            onclick={() => selectOption(option)}
            disabled={option.disabled}
            role="option"
            aria-selected={option.value === value}
          >
            {option.label}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  {#if error}
    <div class="select-error">{error}</div>
  {/if}
</div>

<style>
  .select-wrapper {
    width: 100%;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    font-size: 0.875rem;
  }

  .select-container {
    position: relative;
    width: 100%;
  }

  .select-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background-color: var(--color-blueprint-paper);
    border: 1px solid var(--color-border-strong);
    font-size: 1rem;
    cursor: pointer;
    transition: border-color var(--transition-fast);
    text-align: left;
  }

  .select-trigger:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  .select-trigger:focus {
    outline: none;
    border-color: var(--color-primary);
    border-width: 2px;
    padding: calc(0.75rem - 1px) calc(1rem - 1px);
  }

  .select-trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .select-trigger:disabled:hover {
    border-color: var(--color-blueprint-line-light);
  }

  .select-trigger[data-open="true"] {
    border-bottom-color: transparent;
  }

  .select-trigger[data-placeholder="true"] .select-value {
    color: var(--color-muted);
  }

  .select-value {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .select-arrow {
    margin-left: 0.5rem;
    font-size: 0.75rem;
    color: var(--color-primary);
    transition: transform var(--transition-fast);
    flex-shrink: 0;
  }

  .select-arrow[data-open="true"] {
    transform: rotate(180deg);
  }

  .select-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: var(--z-dropdown);
    background-color: var(--color-blueprint-paper);
    border: 1px solid var(--color-border-strong);
    border-top: 0;
    max-height: 18.75rem;
    overflow-y: auto;
  }

  .select-option {
    width: 100%;
    padding: 0.625rem 0.875rem;
    border: 0;
    background-color: var(--color-blueprint-paper);
    text-align: left;
    cursor: pointer;
    font-size: 0.875rem;
    transition: background-color var(--transition-fast);
  }

  .select-option:hover:not(:disabled) {
    background-color: var(--color-blueprint-line-faint);
  }

  .select-option:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .select-option:disabled:hover {
    background-color: var(--color-blueprint-paper);
  }

  .select-option[data-selected="true"] {
    background-color: var(--color-primary);
    color: white;
    font-weight: 600;
  }

  .select-option[data-selected="true"]:hover {
    background-color: var(--color-primary);
  }

  .select-error {
    margin-top: 0.5rem;
    color: var(--color-error);
    font-size: 0.875rem;
  }
</style>
