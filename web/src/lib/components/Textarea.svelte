<script lang="ts">
  let {
    value = $bindable(""),
    placeholder = "",
    required = false,
    disabled = false,
    id = "",
    class: className = "",
    ...rest
  } = $props<{
    value?: string;
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
    id?: string;
    class?: string;
    [key: string]: any;
  }>();
</script>

<textarea
  bind:value
  {placeholder}
  {required}
  {disabled}
  {id}
  class={`w-full px-4 py-3 border border-border-strong text-base bg-blueprint-paper transition-all resize-y min-h-25 font-mono focus:outline-none focus:border-primary focus:ring-[3px] focus:ring-blue-500/10 ${className}`.trim()}
  {...rest}
></textarea>
