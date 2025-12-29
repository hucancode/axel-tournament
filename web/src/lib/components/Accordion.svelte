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

<details class:disabled {open}>
  <summary>
    <span>▸</span>
    {title}
  </summary>
  <div>
    {@render children()}
  </div>
</details>

<style>
  details {
    border: 1px solid var(--blueprint-line-light);
    background-color: var(--white);
    overflow: hidden;
  }

  details.disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  summary {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    user-select: none;
    list-style: none;
    background-color: var(--blueprint-line-faint);
    background-image: repeating-linear-gradient(315deg, var(--pattern-fg) 0, var(--pattern-fg) 1px, transparent 0, transparent 50%);
    background-size: 10px 10px;
    border-bottom: 1px solid transparent;
    color: var(--text);
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  summary::-webkit-details-marker {
    display: none;
  }

  summary:hover {
    background-color: var(--blueprint-line-faint);
    border-color: var(--primary);
    border-left: 2px solid var(--primary);
    padding-left: calc(1rem - 1px);
  }

  summary:active {
    opacity: 0.9;
  }

  span {
    display: inline-block;
    font-size: 1rem;
    font-weight: bold;
    color: var(--primary);
    transition: transform 0.2s;
  }

  details[open] span {
    transform: rotate(90deg);
  }

  details[open] summary {
    border-bottom-color: var(--blueprint-line-faint);
  }

  div {
    padding: 1rem;
    background-color: var(--white);
  }
</style>
