<script lang="ts">
  export let title: string;
  export let onclose: () => void = () => {};
  export let dialog: HTMLDialogElement | null = null;
</script>

<dialog bind:this={dialog} onclose={onclose}>
  <form method="dialog">
    <header>
      <h2>{title}</h2>
      <button type="button" onclick={() => dialog?.close()} aria-label="Close">×</button>
    </header>
    <div>
      <slot />
    </div>
    <footer>
      <button type="button" onclick={() => dialog?.close()}>Cancel</button>
      <button type="submit" value="submit">Submit</button>
    </footer>
  </form>
</dialog>

<style>
  dialog {
    border: 1px solid var(--blueprint-line-light);
    max-width: 500px;
    width: 90%;
    background: var(--white);
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    margin: 0;
  }

  dialog::backdrop {
    background: rgba(15, 23, 42, 0.6);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid var(--blueprint-line-faint);
    color: var(--text);
  }

  header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  header button {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    font-weight: bold;
    color: var(--text-muted);
    transition: color 0.15s ease;
  }

  header button:hover {
    color: var(--primary);
  }

  /* Modal body - the div between header and footer */
  form > div {
    padding: 1.5rem;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 1px solid var(--blueprint-line-faint);
  }

  footer button {
    padding: 0.75rem 1.5rem;
    border: 1px solid var(--blueprint-line-light);
    background: var(--white);
    color: var(--text);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  footer button:hover {
    border-color: var(--primary);
    background: var(--blueprint-line-faint);
  }

  footer button[type="submit"] {
    background: var(--primary);
    color: var(--white);
    border-color: var(--primary);
  }

  footer button[type="submit"]:hover {
    opacity: 0.9;
    background: var(--primary);
  }
</style>
