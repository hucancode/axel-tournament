<script lang="ts">
  interface Props {
    label?: string;
    value?: string;
    min?: string;
    max?: string;
    disabled?: boolean;
    error?: string;
  }

  let {
    label = 'Date',
    value = $bindable(''),
    min,
    max,
    disabled = false,
    error = ''
  }: Props = $props();

  const inputId = `date-picker-${Math.random().toString(36).slice(2)}`;
</script>

<div class="form-group">
  <label for={inputId}>{label}</label>
  <input
    id={inputId}
    type="date"
    class="input"
    bind:value
    {min}
    {max}
    {disabled}
  />
  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>
