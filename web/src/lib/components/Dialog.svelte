<script lang="ts">
  export let title: string;
  export let onclose: () => void = () => {};
  export let dialog: HTMLDialogElement | null = null;
</script>

<dialog bind:this={dialog} onclose={onclose} class="border border-blueprint-line-light max-w-[500px] w-[90%] bg-blueprint-paper fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 m-0 backdrop:bg-slate-900/60">
  <form method="dialog">
    <header class="flex justify-between items-center p-6 border-b border-blueprint-line-faint">
      <h2 class="m-0 text-xl font-semibold">{title}</h2>
      <button type="button" onclick={() => dialog?.close()} aria-label="Close" class="bg-transparent border-0 text-2xl cursor-pointer font-bold-muted transition-colors hover:text-primary">×</button>
    </header>
    <div class="p-6">
      <slot />
    </div>
    <footer class="flex justify-end gap-4 p-6 border-t border-blueprint-line-faint">
      <button type="button" onclick={() => dialog?.close()} class="px-6 py-3 border border-blueprint-line-light bg-blueprint-paper cursor-pointer font-medium transition-all hover:border-primary hover:bg-blueprint-line-faint">Cancel</button>
      <button type="submit" value="submit" class="px-6 py-3 border border-primary bg-primary text-white cursor-pointer font-medium transition-all hover:opacity-90">Submit</button>
    </footer>
  </form>
</dialog>
