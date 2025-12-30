<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import Statistic from "./Statistic.svelte";

    const { Story } = defineMeta({
        title: "Data Display/Statistic",
        component: Statistic,
    });
</script>

<Story name="Default" args={{ value: 1234, label: "Total Users" }} />
<Story
    name="Positive"
    args={{ value: 567, label: "Active Sessions", variant: "positive" }}
/>
<Story
    name="Negative"
    args={{ value: 89, label: "Failed Logins", variant: "positive" }}
/>
