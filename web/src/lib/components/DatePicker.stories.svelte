<script context="module">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import DatePicker from './DatePicker.svelte';

  const { Story } = defineMeta({
    component: DatePicker,
    title: 'Components/DatePicker',
  });
</script>

<Story name="Default">
  <DatePicker label="Select Date" />
</Story>

<Story name="With Value">
  <DatePicker label="Birth Date" value="2024-01-15" />
</Story>

<Story name="With Min/Max">
  <DatePicker
    label="Tournament Date"
    value="2024-06-15"
    min="2024-01-01"
    max="2024-12-31"
  />
</Story>

<Story name="Disabled">
  <DatePicker label="Locked Date" value="2024-03-20" disabled={true} />
</Story>

<Story name="With Error">
  <DatePicker
    label="Event Date"
    value="2024-02-14"
    error="This date is not available"
  />
</Story>
