<script lang="ts">
  import { type Snippet } from 'svelte';

  interface Props {
    title: string;
    open?: boolean;
    disabled?: boolean;
    children: Snippet;
  }

  let { title, open = false, disabled = false, children }: Props = $props();
</script>

<details class="accordion" class:disabled {open}>
  <summary class="accordion-summary">
    <span class="accordion-icon">▸</span>
    <span class="accordion-title">{title}</span>
  </summary>
  <div class="accordion-content">
    {@render children()}
  </div>
</details>

<style>
  .accordion {
    border: 3px solid var(--black);
    border-radius: 4px;
    background-color: var(--white);
    box-shadow: 3px 3px 0 0 var(--black);
    overflow: hidden;
  }

  .accordion.disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .accordion-summary {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    font-weight: 700;
    font-size: 1rem;
    cursor: pointer;
    user-select: none;
    list-style: none;
    background-color: var(--gray-light);
    border-bottom: 3px solid transparent;
    transition: all 0.1s;
  }

  .accordion-summary::-webkit-details-marker {
    display: none;
  }

  .accordion-summary:hover {
    background-color: var(--primary);
    color: var(--white);
  }

  .accordion-summary:active {
    transform: translate(1px, 1px);
  }

  .accordion-icon {
    display: inline-block;
    font-size: 1rem;
    font-weight: bold;
    transition: transform 0.2s;
  }

  .accordion[open] .accordion-icon {
    transform: rotate(90deg);
  }

  .accordion[open] .accordion-summary {
    border-bottom-color: var(--black);
  }

  .accordion-title {
    flex: 1;
  }

  .accordion-content {
    padding: 1rem;
    background-color: var(--white);
  }
</style>
