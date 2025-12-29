<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Checkbox from './Checkbox.svelte';

  const { Story } = defineMeta({
    title: 'Data Entry/Checkbox',
    component: Checkbox,
  });
</script>

<Story name="Default" args={{ label: 'Accept terms and conditions' }} />

<Story name="Checked" args={{ label: 'Accept terms and conditions', checked: true }} />

<Story name="Disabled" args={{ label: 'Accept terms and conditions', disabled: true }} />

<Story name="Disabled Checked" args={{ label: 'Accept terms and conditions', disabled: true, checked: true }} />

<Story name="Indeterminate" args={{ label: 'Select all', indeterminate: true }} />

<Story name="Without Label" args={{ checked: false }} />
