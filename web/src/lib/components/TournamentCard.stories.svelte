<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import TournamentCard from './TournamentCard.svelte';

  const { Story } = defineMeta({
    title: 'Components/TournamentCard',
    component: TournamentCard,
  });
</script>

<Story name="Default" />
