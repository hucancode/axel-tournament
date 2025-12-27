<script context="module">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import TimePicker from './TimePicker.svelte';

  const { Story } = defineMeta({
    component: TimePicker,
    title: 'Components/TimePicker',
  });
</script>

<Story name="Default">
  <TimePicker label="Select Time" />
</Story>

<Story name="With Value">
  <TimePicker label="Start Time" value="14:30" />
</Story>

<Story name="With Min/Max">
  <TimePicker
    label="Meeting Time"
    value="10:00"
    min="09:00"
    max="17:00"
  />
</Story>

<Story name="With Step">
  <TimePicker
    label="Appointment Time"
    value="12:00"
    step={900}
  />
</Story>

<Story name="Disabled">
  <TimePicker label="Locked Time" value="15:45" disabled={true} />
</Story>

<Story name="With Error">
  <TimePicker
    label="Deadline Time"
    value="23:59"
    error="This time slot is already booked"
  />
</Story>
