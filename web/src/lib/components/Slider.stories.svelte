<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import Slider from "./Slider.svelte";

    const { Story } = defineMeta({
        title: "Data Entry/Slider",
        component: Slider,
    });
</script>

<Story name="Default" args={{ value: 50 }} />

<Story name="With Label" args={{ value: 75, label: "Volume" }} />

<Story
    name="Without Value Display"
    args={{ value: 30, label: "Brightness", showValue: false }}
/>

<Story
    name="Custom Range"
    args={{ value: 5, min: 0, max: 10, label: "Rating" }}
/>

<Story
    name="With Step"
    args={{ value: 50, min: 0, max: 100, step: 10, label: "Percentage" }}
/>

<Story
    name="Disabled"
    args={{ value: 60, label: "Disabled Slider", disabled: true }}
/>
