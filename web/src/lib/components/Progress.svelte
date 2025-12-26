<script lang="ts">
  interface Props {
    value: number;
    max?: number;
    variant?: 'primary' | 'secondary' | 'success' | 'error' | 'accent';
  }

  let { value, max = 100, variant = 'primary' }: Props = $props();
</script>

<progress {value} {max} class="progress progress-{variant}"></progress>

<style>
  .progress {
    width: 100%;
    height: 2rem;
    border: 3px solid var(--black);
    border-radius: 4px;
    box-shadow: 4px 4px 0 0 var(--black);
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
  }

  /* Remove default styling */
  .progress::-webkit-progress-bar {
    background-color: var(--white);
    border-radius: 2px;
  }

  .progress::-moz-progress-bar {
    border-radius: 2px;
  }

  .progress::-webkit-progress-value {
    border-radius: 2px;
  }

  /* Primary variant */
  .progress-primary::-webkit-progress-value {
    background-color: var(--primary);
  }

  .progress-primary::-moz-progress-bar {
    background-color: var(--primary);
  }

  /* Secondary variant */
  .progress-secondary::-webkit-progress-value {
    background-color: var(--secondary);
  }

  .progress-secondary::-moz-progress-bar {
    background-color: var(--secondary);
  }

  /* Success variant */
  .progress-success::-webkit-progress-value {
    background-color: var(--success);
  }

  .progress-success::-moz-progress-bar {
    background-color: var(--success);
  }

  /* Error variant */
  .progress-error::-webkit-progress-value {
    background-color: var(--error);
  }

  .progress-error::-moz-progress-bar {
    background-color: var(--error);
  }

  /* Accent variant */
  .progress-accent::-webkit-progress-value {
    background-color: var(--accent);
  }

  .progress-accent::-moz-progress-bar {
    background-color: var(--accent);
  }
</style>
