<script lang="ts">
    import type { Snippet } from "svelte";

    interface Column {
        key: string;
        label: string;
        align?: "left" | "right" | "center";
    }

    interface Props {
        columns?: Column[];
        children: Snippet;
        class?: string;
    }

    let { columns, children }: Props = $props();
</script>

<div class="container">
    <table>
        {#if columns}
            <thead>
                <tr>
                    {#each columns as col}
                        <th style:text-align={col.align || "left"}>{col.label}</th>
                    {/each}
                </tr>
            </thead>
        {/if}
        <tbody>
            {@render children()}
        </tbody>
    </table>
</div>

<style>
    .container {
        overflow-x: auto;
    }

    table {
        width: 100%;
        border-collapse: collapse;
    }

    thead {
        background-color: var(--color-bg-light);
    }

    :global(.data-table tbody tr:hover) {
        background-color: var(--color-border-light);
    }
</style>
