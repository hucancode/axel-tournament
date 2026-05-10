<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import Statistic from "./Statistic.svelte";

    const { Story } = defineMeta({
        title: "Data Display/Statistic",
        component: Statistic,
        parameters: {
            layout: "fullscreen",
        },
    });
</script>

<Story name="All Variants">
    {#snippet template()}
        <section class="showcase">
            <h2>Statistic</h2>
            <p class="lede">Big number + label. <code>variant</code> tints the number.</p>

            <div class="grid">
                <article class="cell">
                    <Statistic value={1234} label="Total Users" variant="neutral" />
                    <code>variant="neutral"</code>
                </article>
                <article class="cell">
                    <Statistic value={567} label="Active Sessions" variant="positive" />
                    <code>variant="positive"</code>
                </article>
                <article class="cell">
                    <Statistic value={89} label="Failed Logins" variant="negative" />
                    <code>variant="negative"</code>
                </article>
                <article class="cell">
                    <Statistic value="98.7%" label="Uptime" variant="positive" />
                    <code>string value</code>
                </article>
                <article class="cell">
                    <Statistic value="—" label="Pending" variant="neutral" />
                    <code>placeholder value</code>
                </article>
            </div>

            <h3>Dashboard row</h3>
            <div class="row">
                <Statistic value={42} label="Tournaments" />
                <Statistic value={1287} label="Matches Played" variant="positive" />
                <Statistic value={3} label="Errors" variant="negative" />
            </div>
        </section>
    {/snippet}
</Story>

<style>
    .showcase {
        padding: var(--spacing-4);
        max-width: 60rem;
    }
    h2 {
        margin: 0 0 var(--spacing-2) 0;
    }
    h3 {
        margin: var(--spacing-6) 0 var(--spacing-3) 0;
        font-size: var(--font-size-md);
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--color-fg-muted);
    }
    .lede {
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-4);
    }
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
        gap: var(--spacing-3);
    }
    .cell {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-2);
    }
    .row {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: var(--spacing-3);
    }
    code {
        font-family: monospace;
        font-size: var(--font-size-xs);
        color: var(--color-fg-muted);
    }
</style>
