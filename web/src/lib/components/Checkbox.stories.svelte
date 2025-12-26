<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Checkbox from './Checkbox.svelte';

  const { Story } = defineMeta({
    title: 'Forms/Checkbox',
    component: Checkbox,
  });
</script>

<Story name="Default" args={{ label: 'Accept terms and conditions' }} />

<Story name="Checked" args={{ label: 'Accept terms and conditions', checked: true }} />

<Story name="Disabled" args={{ label: 'Accept terms and conditions', disabled: true }} />

<Story name="Disabled Checked" args={{ label: 'Accept terms and conditions', disabled: true, checked: true }} />

<Story name="Indeterminate" args={{ label: 'Select all', indeterminate: true }} />

<Story name="Without Label" args={{ checked: false }} />

<Story name="All States">
  <div class="flex flex-col gap-4">
    <Checkbox label="Unchecked" checked={false} />
    <Checkbox label="Checked" checked={true} />
    <Checkbox label="Indeterminate" indeterminate={true} />
    <Checkbox label="Disabled" disabled={true} />
    <Checkbox label="Disabled Checked" disabled={true} checked={true} />
    <Checkbox checked={false} />
  </div>
</Story>

<Story name="Group Example">
  <div class="flex flex-col gap-4">
    <h3 class="font-bold text-lg mb-2">Select your preferences:</h3>
    <Checkbox label="Receive email notifications" checked={true} />
    <Checkbox label="Receive SMS notifications" checked={false} />
    <Checkbox label="Receive push notifications" checked={true} />
    <Checkbox label="Marketing emails" checked={false} />
  </div>
</Story>
