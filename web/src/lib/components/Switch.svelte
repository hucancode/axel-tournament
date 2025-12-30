<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    onchange?: (checked: boolean) => void;
  }

  let { checked = false, disabled = false, label = '', onchange }: Props = $props();

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
    onchange={handleChange}
    class="absolute opacity-0 w-0 h-0 peer"
  />
  <span class="relative inline-block w-12 h-6 bg-gray-light border border-blueprint-line-light transition-all peer-checked:bg-primary peer-checked:border-primary peer-focus:outline-2 peer-focus:outline-primary peer-focus:outline-offset-2 hover:border-primary hover:border-2 active:opacity-90 {disabled ? 'border-gray-medium' : ''}">
    <span class="absolute top-0.5 left-0.5 w-4.5 h-4.5 bg-blueprint-paper border border-blueprint-line-light transition-transform peer-checked:translate-x-6 peer-checked:bg-blueprint-paper peer-checked:border-white"></span>
  </span>
  {#if label}
    <span class="font-medium">{label}</span>
  {/if}
</label>
