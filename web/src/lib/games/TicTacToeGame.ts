import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext } from './BasePixiGame';
import { COLORS } from './types';

/** Spec: judge/protocols/tic-tac-toe.md. Player 0 = X, player 1 = O. */
export class TicTacToeGame extends BasePixiGame {
  private board: ('X' | 'O' | null)[] = Array(9).fill(null);
  private moveCount = 0;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;

    switch (kind) {
      case 'GAME_STARTED':
        this.board = Array(9).fill(null);
        this.moveCount = 0;
        this.gameState.status = 'playing';
        this.gameState.result = undefined;
        this.render();
        break;
      case 'MOVE': {
        const parts = payload.split(' ');
        const moverIdx = ctx.players.indexOf(parts[0]);
        const row = parseInt(parts[1], 10);
        const col = parseInt(parts[2], 10);
        if (moverIdx < 0 || isNaN(row) || isNaN(col)) return;
        this.board[row * 3 + col] = moverIdx === 0 ? 'X' : 'O';
        this.moveCount += 1;
        this.render();
        break;
      }
      case 'WINNER': {
        const w = parseInt(payload, 10);
        this.gameState.status = 'finished';
        this.gameState.result = w === ctx.myIndex ? 'You Win!' : 'You Lose';
        this.render();
        break;
      }
      case 'DRAW':
        this.gameState.status = 'finished';
        this.gameState.result = 'Draw';
        this.render();
        break;
    }
  }

  protected render(): void {
    this.container.removeChildren();

    const me = this.ctx.myIndex;
    const isMyTurn = this.gameState.status === 'playing' && me >= 0 && this.moveCount % 2 === me;

    for (let i = 0; i < 9; i++) {
      const row = Math.floor(i / 3);
      const col = i % 3;
      const x = col * 120 + 40;
      const y = row * 120 + 80;

      const cell = new Graphics();
      cell.rect(x, y, 100, 100);
      cell.fill(COLORS.WHITE);
      cell.stroke({ width: 2, color: COLORS.BLACK });

      if (this.gameState.status === 'playing' && !this.board[i] && isMyTurn) {
        cell.interactive = true;
        cell.cursor = 'pointer';
        cell.on('pointerdown', () => this.makeMove(row, col));
      }
      this.container.addChild(cell);

      if (this.board[i]) {
        const text = new Text({
          text: this.board[i]!,
          style: { fontSize: 48, fill: COLORS.BLACK },
        });
        text.x = x + 50 - text.width / 2;
        text.y = y + 50 - text.height / 2;
        this.container.addChild(text);
      }
    }

    const status = new Text({
      text: this.getStatusText(isMyTurn),
      style: { fontSize: 16, fill: COLORS.BLACK },
    });
    status.x = 200 - status.width / 2;
    status.y = 20;
    this.container.addChild(status);
  }

  private makeMove(row: number, col: number): void {
    if (!this.wsConnected || this.gameState.status !== 'playing') return;
    const me = this.ctx.myIndex;
    if (me < 0 || this.moveCount % 2 !== me) return;
    if (this.board[row * 3 + col]) return;
    this.sendMove(`${row} ${col}`);
  }

  private getStatusText(isMyTurn: boolean): string {
    if (this.gameState.status === 'waiting') return 'Waiting for game...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    const sym = this.ctx.myIndex === 0 ? 'X' : 'O';
    return isMyTurn ? `Your turn (${sym})` : "Opponent's turn";
  }
}
