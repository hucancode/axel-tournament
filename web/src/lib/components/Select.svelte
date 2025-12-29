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

<div class="w-full">
  {#if label}
    <label class="block mb-2 font-semibold text-sm" for={inputId}>{label}</label>
  {/if}

  <div
    class="relative w-full"
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
      class="w-full flex items-center justify-between px-4 py-3 border border-blueprint-line-light bg-blueprint-paper text-base cursor-pointer transition-colors text-left hover:border-primary focus:outline-none focus:border-primary focus:border-2 focus:px-3.75 focus:py-2.75 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:border-blueprint-line-light {isOpen ? 'border-b-transparent' : ''} {!selectedOption ? 'text-muted' : ''}"
      onclick={toggleDropdown}
      {disabled}
    >
      <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{displayText}</span>
      <span class="ml-2 text-xs text-primary transition-transform shrink-0 {isOpen ? 'rotate-180' : ''}">▼</span>
    </button>

    {#if isOpen}
      <div id={dropdownId} class="absolute top-full left-0 right-0 z-50 bg-blueprint-paper border border-blueprint-line-light border-t-0 max-h-75 overflow-y-auto" role="listbox">
        {#each options as option}
          <button
            type="button"
            class="w-full px-3.5 py-2.5 border-0 bg-blueprint-paper text-left cursor-pointer text-sm transition-colors hover:bg-blueprint-line-faint disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blueprint-paper {option.value === value ? 'bg-primary text-white font-semibold hover:bg-primary' : ''}"
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
    <div class="mt-2 text-error text-sm">{error}</div>
  {/if}
</div>
