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
    <span class="icon">▸</span>
    <span class="title">{title}</span>
  </summary>
  <div>
    {@render children()}
  </div>
</details>

<style>
  details {
    border: 3px solid var(--black);
    border-radius: 4px;
    background-color: var(--white);
    box-shadow: 3px 3px 0 0 var(--black);
    overflow: hidden;
  }

  details.disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  details > summary {
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

  details > summary::-webkit-details-marker {
    display: none;
  }

  details > summary:hover {
    background-color: var(--primary);
    color: var(--white);
  }

  details > summary:active {
    transform: translate(1px, 1px);
  }

  details > summary > .icon {
    display: inline-block;
    font-size: 1rem;
    font-weight: bold;
    transition: transform 0.2s;
  }

  details[open] > summary > .icon {
    transform: rotate(90deg);
  }

  details[open] > summary {
    border-bottom-color: var(--black);
  }

  details > summary > .title {
    flex: 1;
  }

  details > div {
    padding: 1rem;
    background-color: var(--white);
  }
</style>
