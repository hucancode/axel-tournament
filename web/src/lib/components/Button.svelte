<script lang="ts">
  interface Props {
    variant?: 'primary' | 'secondary' | 'danger' | 'success';
    disabled?: boolean;
    label?: string;
  }

  let { variant = 'primary', disabled = false, label = 'Button' }: Props = $props();

  let className = $derived(`btn btn-${variant}`);
</script>

<button class={className} {disabled}>
  {label}
</button>
