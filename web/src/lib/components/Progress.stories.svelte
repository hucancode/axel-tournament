<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import Progress from "./Progress.svelte";

    const { Story } = defineMeta({
        title: "Data Display/Progress",
        component: Progress,
    });
</script>

<Story name="In Progress">
    {#snippet template()}
        <div style="display: flex; flex-direction: column; gap: 1rem;">
            <Progress value={75} variant="primary" />
            <Progress value={60} variant="secondary" />
            <Progress value={100} variant="success" />
            <Progress value={30} variant="error" />
            <Progress value={45} variant="accent" />
        </div>
    {/snippet}
</Story>
<Story name="Indeterminate">
    {#snippet template()}
        <div style="display: flex; flex-direction: column; gap: 1rem;">
            <Progress variant="primary" />
            <Progress variant="secondary" />
            <Progress variant="success" />
            <Progress variant="error" />
            <Progress variant="accent" />
        </div>
    {/snippet}
</Story>
<Story name="With Label">
    {#snippet template()}
        <div style="display: flex; flex-direction: column; gap: 1rem;">
            <Progress value={65} labelPosition="middle" />
            <Progress value={75} labelPosition="top" />
            <Progress value={45} labelPosition="right" />
        </div>
    {/snippet}
</Story>
