<script lang="ts">
    import Button from "./Button.svelte";
    import type { Snippet } from "svelte";

    interface Props {
        title: string;
        onclose?: () => void;
        dialog?: HTMLDialogElement | null;
        children?: Snippet;
    }
    let {
        title,
        onclose = () => {},
        dialog = $bindable(null),
        children,
    }: Props = $props();
</script>

<dialog
    bind:this={dialog}
    {onclose}
    class="max-w-125 w-[90%] bg-blueprint-paper fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 backdrop:bg-slate-900/60"
>
    <form method="dialog">
        <header
            class="flex justify-between items-center p-6 border-b border-blueprint-line-faint"
        >
            <h2 class="text-xl font-semibold">{title}</h2>
            <button
                type="button"
                onclick={() => dialog?.close()}
                aria-label="Close"
                class="text-2xl cursor-pointer hover:text-primary">×</button
            >
        </header>
        {#if children}
            <div class="p-6">
                {@render children()}
            </div>
        {/if}
        <footer
            class="flex justify-end gap-4 p-6 border-t border-blueprint-line-faint"
        >
            <Button
                variant="secondary"
                type="button"
                label="Cancel"
                onclick={() => dialog?.close()}
            />
            <Button variant="primary" type="submit" label="Submit" />
        </footer>
    </form>
</dialog>
