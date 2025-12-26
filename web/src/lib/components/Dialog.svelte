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
    border: 4px solid #000;
    border-radius: 4px;
    padding: 0;
    max-width: 500px;
    width: 90%;
    box-shadow: 8px 8px 0 0 #000;
    background: #fff;
  }

  dialog::backdrop {
    background: rgba(0, 0, 0, 0.8);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 2px solid #000;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    font-weight: bold;
  }

  .modal-body {
    padding: 1.5rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 2px solid #000;
  }
</style>
