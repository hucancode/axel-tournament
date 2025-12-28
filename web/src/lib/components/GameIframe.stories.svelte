<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import GameIframe from './GameIframe.svelte';
  import { defaultInteractiveFrontend } from './templates/game-templates';

  const { Story } = defineMeta({
    title: 'Data Display/GameIframe',
    component: GameIframe,
    args: {
      gameCode: "",
      roomId: 'room-123',
      wsUrl: 'ws://localhost:8080/ws',
    },
  });
</script>

<Story name="Default" />

<Story name="TicTacToe" args={{ gameCode: defaultInteractiveFrontend }} />
