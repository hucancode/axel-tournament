<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    indeterminate?: boolean;
    onchange?: (checked: boolean) => void;
  }

  let { checked = false, disabled = false, label = '', indeterminate = false, onchange }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    if (onchange) {
      onchange(target.checked);
    }
  }
</script>

<label class="inline-flex items-center gap-3 cursor-pointer select-none {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
  <input
    type="checkbox"
    bind:checked
    {disabled}
    {indeterminate}
    onchange={handleChange}
    class="absolute opacity-0 w-0 h-0 peer"
  />
  <span class="relative inline-flex items-center justify-center w-5 h-5 bg-blueprint-paper border border-blueprint-line-light transition-colors peer-checked:bg-primary peer-checked:border-primary peer-indeterminate:bg-blueprint-line-light peer-indeterminate:border-blueprint-line-light peer-focus:outline-2 peer-focus:outline-primary peer-focus:outline-offset-2 hover:border-primary hover:border-2 active:opacity-90 {disabled ? 'border-gray-medium' : ''}">
    {#if indeterminate}
      <span class="w-2.5 h-0.5 bg-white"></span>
    {:else if checked}
      <svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 text-white">
        <path d="M2 8L6 12L14 4" stroke="currentColor" stroke-width="3" stroke-linecap="square" stroke-linejoin="miter"/>
      </svg>
    {/if}
  </span>
  {#if label}
    <span class="font-medium">{label}</span>
  {/if}
</label>
