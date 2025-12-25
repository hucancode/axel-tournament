<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import PageHeader from './PageHeader.svelte';

  const { Story } = defineMeta({
    title: 'Components/PageHeader',
    component: PageHeader,
    args: {
      title: 'Welcome to Axel Tournament',
      subtitle: 'Compete in coding tournaments, submit your AI bots, and climb the leaderboard',
      centered: false,
    },
  });
</script>

<Story name="Default" />

<Story name="Centered" args={{ title: 'Tournaments', subtitle: 'Browse and join programming tournaments', centered: true }} />

<Story name="TitleOnly" args={{ title: 'Profile', subtitle: undefined }} />
