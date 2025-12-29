<script lang="ts">
  let {
    type = "text",
    value = $bindable(""),
    placeholder = "",
    required = false,
    disabled = false,
    id = "",
    class: className = "",
    ...rest
  } = $props<{
    type?: string;
    value?: string;
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
    id?: string;
    class?: string;
    [key: string]: any;
  }>();
</script>

<input
  {type}
  bind:value
  {placeholder}
  {required}
  {disabled}
  {id}
  class={`w-full px-4 py-3 border border-border-strong text-base bg-blueprint-paper transition-all focus:outline-none focus:border-primary focus:ring-[3px] focus:ring-blue-500/10 ${className}`.trim()}
  {...rest}
/>
