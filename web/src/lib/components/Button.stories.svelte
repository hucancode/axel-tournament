<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Button from './Button.svelte';

  const { Story } = defineMeta({
    title: 'Data Entry/Button',
    component: Button,
  });
</script>

<Story name="Primary" args={{ variant: 'primary', label: 'Primary Button' }} />

<Story name="Secondary" args={{ variant: 'secondary', label: 'Secondary Button' }} />

<Story name="Danger" args={{ variant: 'danger', label: 'Delete' }} />

<Story name="Success" args={{ variant: 'success', label: 'Confirm' }} />

<Story name="Disabled" args={{ variant: 'primary', label: 'Disabled Button', disabled: true }} />
