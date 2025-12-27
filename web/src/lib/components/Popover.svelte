<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        trigger: Snippet;
        children: Snippet;
        id?: string;
        disabled?: boolean;
    }

    let {
        trigger,
        children,
        id = `popover-${Math.random().toString(36).substring(2, 11)}`,
        disabled = false,
    }: Props = $props();
    let triggerButton: HTMLButtonElement;
    let popoverElement: HTMLElement;
</script>

<button
    class="btn inline-block"
    bind:this={triggerButton}
    {disabled}
    popovertarget={id}
>
    {@render trigger()}
</button>
<div {id} bind:this={popoverElement} popover>
    {@render children()}
</div>
