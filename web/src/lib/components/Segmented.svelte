<script lang="ts">
  interface SegmentedOption {
    value: string;
    label: string;
  }

  interface Props {
    options: SegmentedOption[];
    value?: string;
    disabled?: boolean;
    name?: string;
    onchange?: (value: string) => void;
  }

  let {
    options,
    value = $bindable(options[0]?.value),
    disabled = false,
    name = `segmented-${Math.random().toString(36).slice(2)}`,
    onchange
  }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    if (onchange) {
      onchange(target.value);
    }
  }
</script>

<div role="radiogroup" class="inline-flex bg-blueprint-paper border border-blueprint-line-light overflow-hidden {disabled ? 'opacity-50 cursor-not-allowed border-gray-medium' : ''}">
  {#each options as option, index (option.value)}
    <label class="px-6 py-3 bg-transparent border-r border-blueprint-line-faint font-medium cursor-pointer transition-all relative inline-flex items-center select-none last:border-r-0 has-checked:bg-primary has-checked:text-white has-checked:border-primary has-focus:outline-2 has-focus:outline-primary has-focus:-outline-offset-2 has-focus:z-10 hover:has-[:not(:disabled):not(:checked)]:bg-blueprint-line-faint active:has-[:not(:disabled)]:bg-primary active:has-[:not(:disabled)]:text-white has-disabled:cursor-not-allowed {disabled ? 'border-gray-medium' : ''}">
      <input
        type="radio"
        {name}
        value={option.value}
        checked={option.value == value}
        onchange={handleChange}
        {disabled}
        class="absolute opacity-0 w-0 h-0"
      />
      {option.label}
    </label>
  {/each}
</div>
