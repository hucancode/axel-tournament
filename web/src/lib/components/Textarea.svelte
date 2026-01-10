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
  class={className}
  {...rest}
></textarea>

<style>
  textarea {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 1px solid var(--color-border-strong);
    font-size: 1rem;
    background-color: var(--color-blueprint-paper);
    color: var(--color-fg);
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
    resize: vertical;
    min-height: 6.25rem;
    font-family: monospace;
  }

  textarea:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgb(59 130 246 / 0.1);
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
