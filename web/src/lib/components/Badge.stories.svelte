<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Badge from './Badge.svelte';

  const { Story } = defineMeta({
    title: 'Data Display/Badge',
    component: Badge,
  });
</script>

<Story name="Scheduled" args={{ status: 'scheduled', label: 'Scheduled' }} />

<Story name="Registration" args={{ status: 'registration', label: 'Registration Open' }} />

<Story name="Generating" args={{ status: 'generating', label: 'Generating Matches' }} />

<Story name="Running" args={{ status: 'running', label: 'In Progress' }} />

<Story name="Completed" args={{ status: 'completed', label: 'Completed' }} />

<Story name="Cancelled" args={{ status: 'cancelled', label: 'Cancelled' }} />

<Story name="Pending" args={{ status: 'pending', label: 'Pending' }} />

<Story name="Accepted" args={{ status: 'accepted', label: 'Accepted' }} />

<Story name="Failed" args={{ status: 'failed', label: 'Failed' }} />
