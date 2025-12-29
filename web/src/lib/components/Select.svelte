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

<div class="select-container">
  {#if label}
    <label class="select-label" for={inputId}>{label}</label>
  {/if}

  <div
    class="select-wrapper"
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
      class:select-trigger-open={isOpen}
      class:select-trigger-disabled={disabled}
      class:select-trigger-placeholder={!selectedOption}
      onclick={toggleDropdown}
      {disabled}
    >
      <span class="select-value">{displayText}</span>
      <span class="select-arrow" class:select-arrow-open={isOpen}>▼</span>
    </button>

    {#if isOpen}
      <div id={dropdownId} class="select-dropdown" role="listbox">
        {#each options as option}
          <button
            type="button"
            class="select-option"
            class:select-option-selected={option.value === value}
            class:select-option-disabled={option.disabled}
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
    <div class="form-error">{error}</div>
  {/if}
</div>

<style>
  .select-container {
    width: 100%;
  }

  .select-label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: var(--text);
    font-size: 0.875rem;
  }

  .select-wrapper {
    position: relative;
    width: 100%;
  }

  .select-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border: 1px solid var(--blueprint-line-light);
    background: var(--white);
    border-radius: 0;
    font-size: 1rem;
    color: var(--text);
    cursor: pointer;
    transition: border-color 0.15s ease;
    text-align: left;
  }

  .select-trigger:hover:not(:disabled) {
    border-color: var(--primary);
  }

  .select-trigger:focus {
    outline: none;
    border-color: var(--primary);
    border-width: 2px;
    padding: calc(0.75rem - 1px) calc(1rem - 1px);
  }

  .select-trigger-open {
    border-bottom-color: transparent;
  }

  .select-trigger-disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .select-trigger-placeholder {
    color: var(--text-muted);
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
    color: var(--primary);
    transition: transform 0.2s;
    flex-shrink: 0;
  }

  .select-arrow-open {
    transform: rotate(180deg);
  }

  .select-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 50;
    background: var(--white);
    border: 1px solid var(--blueprint-line-light);
    border-top: none;
    border-radius: 0;
    max-height: 300px;
    overflow-y: auto;
  }

  .select-option {
    width: 100%;
    padding: 0.625rem 0.875rem;
    border: none;
    background: var(--white);
    text-align: left;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--text);
    transition: background-color 0.15s ease;
  }

  .select-option:hover:not(:disabled) {
    background: var(--blueprint-line-faint);
  }

  .select-option-selected {
    background: var(--primary);
    color: var(--white);
    font-weight: 600;
  }

  .select-option-disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-error {
    margin-top: 0.5rem;
    color: var(--error);
    font-size: 0.875rem;
  }
</style>
