<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import PlayerCount from './PlayerCount.svelte';

  const { Story } = defineMeta({
    title: 'Data Display/PlayerCount',
    component: PlayerCount,
    args: {
      current: 5,
      min: 4,
      max: 10,
    },
  });
</script>

<Story name="Active" />

<Story name="Full" args={{ current: 10 }} />

<Story name="NeedMore" args={{ current: 2 }} />
