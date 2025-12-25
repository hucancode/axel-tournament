import type { Meta, StoryObj } from '@storybook/svelte';
import GameIframe from './GameIframe.svelte';

const meta = {
  title: 'Components/GameIframe',
  component: GameIframe,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof GameIframe>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    gameCode: '<h1>Sample Game</h1><p>This is a demo game interface.</p>',
    roomId: 'demo-room-123',
    wsUrl: 'ws://localhost:3000',
  },
};

export const TicTacToe: Story = {
  args: {
    gameCode: `
      <style>
        .board { display: grid; grid-template-columns: repeat(3, 100px); gap: 2px; margin: 20px auto; width: 306px; }
        .cell { width: 100px; height: 100px; background: #f0f0f0; border: 1px solid #ccc; display: flex; align-items: center; justify-content: center; font-size: 24px; cursor: pointer; }
        .cell:hover { background: #e0e0e0; }
      </style>
      <div id="status">Waiting for game to start...</div>
      <div class="board" id="board"></div>
      <script>
        for(let i=0; i<9; i++) {
          const cell = document.createElement('div');
          cell.className = 'cell';
          cell.onclick = () => window.gameAPI?.sendMove('MOVE ' + i);
          document.getElementById('board').appendChild(cell);
        }
      </script>
    `,
    roomId: 'tictactoe-room-456',
    wsUrl: 'ws://localhost:3000',
  },
};
