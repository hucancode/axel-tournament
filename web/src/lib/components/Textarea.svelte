<script lang="ts">
  interface Props {
    label?: string;
    placeholder?: string;
    value?: string;
    error?: string;
    rows?: number;
    disabled?: boolean;
  }

  let {
    label,
    placeholder = '',
    value = $bindable(''),
    error = '',
    rows = 4,
    disabled = false
  }: Props = $props();

  const textareaId = `textarea-${Math.random().toString(36).slice(2)}`;
</script>

<div class="form-group">
  {#if label}
    <label for={textareaId}>{label}</label>
  {/if}
  <textarea
    id={textareaId}
    class="textarea"
    {placeholder}
    {rows}
    {disabled}
    bind:value
  ></textarea>
  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>
