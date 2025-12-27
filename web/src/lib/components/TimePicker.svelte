<script lang="ts">
  interface Props {
    label?: string;
    value?: string;
    min?: string;
    max?: string;
    step?: number;
    disabled?: boolean;
    error?: string;
  }

  let {
    label = 'Time',
    value = $bindable(''),
    min,
    max,
    step,
    disabled = false,
    error = ''
  }: Props = $props();

  const inputId = `time-picker-${Math.random().toString(36).slice(2)}`;
</script>

<div class="form-group">
  <label for={inputId}>{label}</label>
  <input
    id={inputId}
    type="time"
    class="input"
    bind:value
    {min}
    {max}
    {step}
    {disabled}
  />
  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>
