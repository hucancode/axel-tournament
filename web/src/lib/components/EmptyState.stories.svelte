<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import EmptyState from './EmptyState.svelte';

  const { Story } = defineMeta({
    title: 'Components/EmptyState',
    component: EmptyState,
  });
</script>

<Story name="Default" />

<Story name="NoTournaments" args={{ message: 'No active tournaments at the moment. Check back soon!' }} />
