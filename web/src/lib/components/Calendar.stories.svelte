<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import Calendar from "./Calendar.svelte";

    const { Story } = defineMeta({
        title: "Data Entry/Calendar",
        component: Calendar,
    });

    const today = new Date();
    const minDate = new Date(today.getFullYear(), today.getMonth(), 1);
    const maxDate = new Date(today.getFullYear(), today.getMonth() + 2, 0);
</script>

<Story name="Default">
    {#snippet template()}
        <Calendar
            value={today}
            onSelect={(d) => console.log("selected", d)}
        />
    {/snippet}
</Story>

<Story name="With Min/Max">
    {#snippet template()}
        <Calendar
            value={today}
            min={minDate}
            max={maxDate}
            onSelect={(d) => console.log("selected", d)}
        />
    {/snippet}
</Story>

<Story name="Disabled">
    {#snippet template()}
        <Calendar value={today} disabled />
    {/snippet}
</Story>
