<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import StatusBadge from './StatusBadge.svelte';

  const { Story } = defineMeta({
    title: 'Components/StatusBadge',
    component: StatusBadge,
  });
</script>

<Story name="Running" />

<Story name="Cancelled" args={{ status: 'cancelled', label: 'Cancelled' }} />
