<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";

    const { Story } = defineMeta({
        title: "Foundations/Design Tokens",
        parameters: {
            layout: "fullscreen",
        },
    });

    const colors = [
        { name: "bg", token: "--color-bg" },
        { name: "bg-dark", token: "--color-bg-dark" },
        { name: "bg-light", token: "--color-bg-light" },
        { name: "bg-popup", token: "--color-bg-popup" },
        { name: "fg", token: "--color-fg" },
        { name: "fg-muted", token: "--color-fg-muted" },
        { name: "fg-dim", token: "--color-fg-dim" },
        { name: "border", token: "--color-border" },
        { name: "border-light", token: "--color-border-light" },
        { name: "primary", token: "--color-primary" },
        { name: "secondary", token: "--color-secondary" },
        { name: "accent", token: "--color-accent" },
        { name: "success", token: "--color-success" },
        { name: "warning", token: "--color-warning" },
        { name: "error", token: "--color-error" },
        { name: "info", token: "--color-info" },
        { name: "pink", token: "--color-pink" },
        { name: "orange", token: "--color-orange" },
        { name: "red", token: "--color-red" },
        { name: "aqua", token: "--color-aqua" },
    ];

    const spacing = [1, 2, 3, 4, 5, 6, 8, 12, 16];
    const fontSizes = ["xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl"];
    const radii = ["none", "sm", "md", "lg", "full"];
    const shadows = ["sm", "md", "lg", "popup"];
    const zIndex = [
        { name: "dropdown", token: "--z-dropdown" },
        { name: "sticky", token: "--z-sticky" },
        { name: "fixed", token: "--z-fixed" },
        { name: "modal-backdrop", token: "--z-modal-backdrop" },
        { name: "modal", token: "--z-modal" },
        { name: "popover", token: "--z-popover" },
        { name: "tooltip", token: "--z-tooltip" },
        { name: "toast", token: "--z-toast" },
    ];
</script>

<Story name="Colors">
    {#snippet template()}
        <section class="grid">
            {#each colors as { name, token }}
                <article class="swatch">
                    <div class="chip" style:background={`var(${token})`}></div>
                    <code>{token}</code>
                    <small>{name}</small>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Spacing">
    {#snippet template()}
        <section class="rows">
            {#each spacing as n}
                <article class="row">
                    <code>--spacing-{n}</code>
                    <div class="bar" style:width={`var(--spacing-${n})`}></div>
                    <small>{n * 0.25}rem</small>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Font Sizes">
    {#snippet template()}
        <section class="rows">
            {#each fontSizes as size}
                <article class="row">
                    <code>--font-size-{size}</code>
                    <p style:font-size={`var(--font-size-${size})`}>
                        The quick brown fox jumps
                    </p>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Radius">
    {#snippet template()}
        <section class="grid grid-narrow">
            {#each radii as r}
                <article class="swatch">
                    <div
                        class="block"
                        style:border-radius={`var(--radius-${r})`}
                    ></div>
                    <code>--radius-{r}</code>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Shadows">
    {#snippet template()}
        <section class="grid grid-narrow">
            {#each shadows as s}
                <article class="swatch">
                    <div
                        class="block solid"
                        style:box-shadow={`var(--shadow-${s})`}
                    ></div>
                    <code>--shadow-{s}</code>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Borders">
    {#snippet template()}
        <section class="rows">
            <article class="row">
                <code>--border-thin</code>
                <div
                    class="border-demo"
                    style:border-width="var(--border-thin)"
                ></div>
                <small>1px</small>
            </article>
            <article class="row">
                <code>--border-thick</code>
                <div
                    class="border-demo"
                    style:border-width="var(--border-thick)"
                ></div>
                <small>3px</small>
            </article>
        </section>
    {/snippet}
</Story>

<Story name="Transitions">
    {#snippet template()}
        <section class="rows">
            <article class="row">
                <code>--transition-fast</code>
                <button class="t-demo t-fast">hover me</button>
                <small>0.15s</small>
            </article>
            <article class="row">
                <code>--transition-normal</code>
                <button class="t-demo t-normal">hover me</button>
                <small>0.20s</small>
            </article>
            <article class="row">
                <code>--transition-slow</code>
                <button class="t-demo t-slow">hover me</button>
                <small>0.30s</small>
            </article>
        </section>
    {/snippet}
</Story>

<Story name="Z-Index Scale">
    {#snippet template()}
        <section class="rows">
            {#each zIndex as { name, token }}
                <article class="row">
                    <code>{token}</code>
                    <small class="z-name">{name}</small>
                </article>
            {/each}
        </section>
    {/snippet}
</Story>

<Story name="Patterns">
    {#snippet template()}
        <section class="grid grid-narrow">
            <article class="swatch">
                <div class="block bg-hatch"></div>
                <code>.bg-hatch</code>
            </article>
            <article class="swatch">
                <div class="block bg-grid"></div>
                <code>.bg-grid</code>
            </article>
            <article class="swatch">
                <div class="block bg-grid-major"></div>
                <code>.bg-grid-major</code>
            </article>
            <article class="swatch">
                <div
                    class="block solid"
                    style:background-image="var(--pattern-disabled)"
                    style:background-size="12px 12px"
                ></div>
                <code>--pattern-disabled</code>
            </article>
        </section>
    {/snippet}
</Story>

<style>
    section {
        padding: var(--spacing-4);
    }

    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(13rem, 1fr));
        gap: var(--spacing-4);
    }

    .grid-narrow {
        grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    }

    .swatch {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-1);
    }

    .chip {
        height: 4rem;
        border: var(--border-thin) solid var(--color-border);
    }

    .block {
        height: 5rem;
        background: var(--color-bg-light);
        border: var(--border-thin) solid var(--color-border);
    }

    .block.solid {
        background: var(--color-primary);
        border: none;
    }

    code {
        font-family: monospace;
        font-size: var(--font-size-xs);
        color: var(--color-fg-muted);
    }

    small {
        font-size: var(--font-size-xs);
        color: var(--color-fg-dim);
    }

    .rows {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-3);
    }

    .row {
        display: grid;
        grid-template-columns: 12rem 1fr 6rem;
        align-items: center;
        gap: var(--spacing-4);
    }

    .bar {
        height: 1.25rem;
        background: var(--color-primary);
    }

    .border-demo {
        height: 2rem;
        width: 8rem;
        border-style: solid;
        border-color: var(--color-primary);
        background: var(--color-bg-light);
    }

    .t-demo {
        transition-property: background, transform;
    }
    .t-fast {
        transition-duration: 0.15s;
    }
    .t-normal {
        transition-duration: 0.2s;
    }
    .t-slow {
        transition-duration: 0.3s;
    }
    .t-demo:hover {
        background: var(--color-primary);
        color: var(--color-bg-dark);
        transform: translate(-2px, -2px);
    }

    .z-name {
        color: var(--color-fg);
    }
</style>
