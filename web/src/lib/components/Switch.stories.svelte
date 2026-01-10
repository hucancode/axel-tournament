<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Switch from './Switch.svelte';

  const { Story } = defineMeta({
    title: 'Data Entry/Switch',
    component: Switch,
  });
</script>

<Story name="Default" args={{ label: 'Enable feature' }} />

<Story name="Checked" args={{ label: 'Enable feature', checked: true }} />

<Story name="Disabled" args={{ label: 'Enable feature', disabled: true }} />

<Story name="Disabled Checked" args={{ label: 'Enable feature', disabled: true, checked: true }} />

<Story name="Without Label" args={{ checked: false }} />

<Story name="All States">
  <section class="states-demo">
    <Switch label="Unchecked" checked={false} />
    <Switch label="Checked" checked={true} />
    <Switch label="Disabled" disabled={true} />
    <Switch label="Disabled Checked" disabled={true} checked={true} />
    <Switch checked={false} />
  </section>
</Story>

<style>
  .states-demo {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
