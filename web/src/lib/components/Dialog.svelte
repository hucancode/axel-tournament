<script lang="ts">
  export let title: string;
  export let onclose: () => void = () => {};
  export let dialog: HTMLDialogElement | null = null;
</script>

<dialog bind:this={dialog} onclose={onclose}>
  <form method="dialog">
    <div class="modal-header">
      <h2>{title}</h2>
      <button type="button" class="close-btn" onclick={() => dialog?.close()}>×</button>
    </div>
    <div class="modal-body">
      <slot />
    </div>
    <div class="modal-footer">
      <button type="button" class="btn-secondary" onclick={() => dialog?.close()}>Cancel</button>
      <button type="submit" class="btn-primary" value="submit">Submit</button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    border: 1px solid var(--blueprint-line-light);
    border-radius: 0;
    padding: 0;
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

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid var(--blueprint-line-faint);
    color: var(--text);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    font-weight: bold;
    color: var(--text-muted);
    transition: color 0.15s ease;
  }

  .close-btn:hover {
    color: var(--primary);
  }

  .modal-body {
    padding: 1.5rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 1px solid var(--blueprint-line-faint);
  }
</style>
