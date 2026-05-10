<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";

    const { Story } = defineMeta({
        title: "Foundations/Icons",
        parameters: {
            layout: "fullscreen",
        },
    });

    const icons = [
        "check",
        "x",
        "chevron-up",
        "chevron-down",
        "chevron-left",
        "chevron-right",
        "arrow-left",
        "arrow-right",
        "calendar",
        "clock",
        "warning",
        "info",
        "search",
        "plus",
        "minus",
        "trash",
        "edit",
        "external",
        "user",
        "trophy",
        "play",
    ];
</script>

<Story name="Sprite Catalog">
    {#snippet template()}
        <section class="usage">
            <h2>Usage</h2>
            <p>
                Icons are stored as a single SVG sprite at <code
                    >/static/icons.svg</code
                >. Reference any symbol with raw HTML:
            </p>
            <pre><code
                    >&lt;svg class="icon"&gt;&lt;use href="/icons.svg#i-check" /&gt;&lt;/svg&gt;</code
                ></pre>
            <p>
                Default size is <code>1em</code> (scales with font-size).
                Override per-instance with <code>--icon-size</code> or width/height.
                Stroke uses <code>currentColor</code>.
            </p>
        </section>

        <section class="grid">
            {#each icons as name}
                <article class="cell">
                    <svg class="icon"><use href="/icons.svg#i-{name}" /></svg>
                    <code>i-{name}</code>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Sizes">
    {#snippet template()}
        <section class="row">
            <span style:font-size="0.75rem">
                <svg class="icon"><use href="/icons.svg#i-check" /></svg>
                inline 0.75rem
            </span>
            <span style:font-size="1rem">
                <svg class="icon"><use href="/icons.svg#i-check" /></svg>
                inline 1rem
            </span>
            <span style:font-size="1.5rem">
                <svg class="icon"><use href="/icons.svg#i-check" /></svg>
                inline 1.5rem
            </span>
            <span style:font-size="2rem">
                <svg class="icon"><use href="/icons.svg#i-check" /></svg>
                inline 2rem
            </span>
            <span>
                <svg class="icon" style:--icon-size="3rem">
                    <use href="/icons.svg#i-check" />
                </svg>
                override 3rem
            </span>
        </section>
    {/snippet}
</Story>

<Story name="Colors">
    {#snippet template()}
        <section class="row">
            <span style:color="var(--color-primary)">
                <svg class="icon" style:--icon-size="2rem">
                    <use href="/icons.svg#i-trophy" />
                </svg>
                primary
            </span>
            <span style:color="var(--color-success)">
                <svg class="icon" style:--icon-size="2rem">
                    <use href="/icons.svg#i-check" />
                </svg>
                success
            </span>
            <span style:color="var(--color-warning)">
                <svg class="icon" style:--icon-size="2rem">
                    <use href="/icons.svg#i-warning" />
                </svg>
                warning
            </span>
            <span style:color="var(--color-error)">
                <svg class="icon" style:--icon-size="2rem">
                    <use href="/icons.svg#i-x" />
                </svg>
                error
            </span>
        </section>
    {/snippet}
</Story>

<Story name="In Buttons">
    {#snippet template()}
        <section class="row">
            <button>
                <svg class="icon"><use href="/icons.svg#i-plus" /></svg>
                Add
            </button>
            <button data-variant="primary">
                <svg class="icon"><use href="/icons.svg#i-check" /></svg>
                Confirm
            </button>
            <button data-variant="danger">
                <svg class="icon"><use href="/icons.svg#i-trash" /></svg>
                Delete
            </button>
            <button data-variant="ghost" aria-label="Edit">
                <svg class="icon"><use href="/icons.svg#i-edit" /></svg>
            </button>
            <button data-variant="ghost" aria-label="Close">
                <svg class="icon"><use href="/icons.svg#i-x" /></svg>
            </button>
        </section>
    {/snippet}
</Story>

<style>
    section {
        padding: var(--spacing-4);
    }

    .usage {
        background: var(--color-bg-light);
        border: var(--border-thin) solid var(--color-border);
        margin-bottom: var(--spacing-4);
    }

    .usage pre {
        background: var(--color-bg-dark);
        padding: var(--spacing-3);
        overflow-x: auto;
        font-size: var(--font-size-sm);
    }

    .usage code {
        font-family: monospace;
    }

    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));
        gap: var(--spacing-3);
    }

    .cell {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-2);
        padding: var(--spacing-3);
        background: var(--color-bg-light);
        border: var(--border-thin) solid var(--color-border);
        font-size: var(--font-size-2xl);
    }

    .cell code {
        font-size: var(--font-size-xs);
        color: var(--color-fg-muted);
        font-family: monospace;
    }

    .row {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: var(--spacing-4);
    }

    .row span {
        display: inline-flex;
        align-items: center;
        gap: var(--spacing-2);
    }

    button {
        display: inline-flex;
        align-items: center;
        gap: var(--spacing-2);
    }
</style>
