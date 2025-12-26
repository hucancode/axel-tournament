<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Button from './Button.svelte';

  const { Story } = defineMeta({
    title: 'Styles/Buttons',
    component: Button,
  });
</script>

<Story name="Primary" args={{ variant: 'primary', label: 'Primary Button' }} />

<Story name="Secondary" args={{ variant: 'secondary', label: 'Secondary Button' }} />

<Story name="Danger" args={{ variant: 'danger', label: 'Delete' }} />

<Story name="Success" args={{ variant: 'success', label: 'Confirm' }} />

<Story name="Disabled" args={{ variant: 'primary', label: 'Disabled Button', disabled: true }} />

<Story name="All Variants">
  <div class="flex flex-col gap-4 items-start">
    <Button variant="primary" label="Primary Button" />
    <Button variant="secondary" label="Secondary Button" />
    <Button variant="danger" label="Danger Button" />
    <Button variant="success" label="Success Button" />
    <Button variant="primary" label="Disabled Button" disabled />
  </div>
</Story>
