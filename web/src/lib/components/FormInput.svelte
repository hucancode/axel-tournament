<script lang="ts">
  interface Props {
    type?: 'text' | 'email' | 'select' | 'textarea';
    label?: string;
    placeholder?: string;
    value?: string;
    error?: string;
    options?: string[];
  }

  let {
    type = 'text',
    label = 'Label',
    placeholder = '',
    value = '',
    error = '',
    options = ['Option 1', 'Option 2', 'Option 3']
  }: Props = $props();

  const inputId = `form-input-${Math.random().toString(36).slice(2)}`;
</script>

<div class="form-group">
  <label for={inputId}>{label}</label>
  {#if type === 'select'}
    <select id={inputId} class="select" {value}>
      {#each options as option}
        <option>{option}</option>
      {/each}
    </select>
  {:else if type === 'textarea'}
    <textarea id={inputId} class="textarea" {placeholder} {value}></textarea>
  {:else}
    <input id={inputId} type={type} class="input" {placeholder} {value} />
  {/if}
  {#if error}
    <div class="form-error">{error}</div>
  {/if}
</div>
