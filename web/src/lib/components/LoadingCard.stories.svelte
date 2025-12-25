<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import LoadingCard from './LoadingCard.svelte';

  const { Story } = defineMeta({
    title: 'Components/LoadingCard',
    component: LoadingCard,
    args: {
      message: 'Loading...',
    },
  });
</script>

<Story name="Default" />

<Story name="CustomMessage" args={{ message: 'Loading tournaments...' }} />
