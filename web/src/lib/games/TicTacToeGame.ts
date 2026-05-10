import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

/** Spec: judge/protocols/tic-tac-toe.md. Player 0 = X, player 1 = O. */
export class TicTacToeGame extends BasePixiGame {
  private board: ('X' | 'O' | null)[] = Array(9).fill(null);
  private moveCount = 0;
  private outcome: 'win' | 'lose' | 'draw' | null = null;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;

    switch (kind) {
      case 'GAME_STARTED':
        this.board = Array(9).fill(null);
        this.moveCount = 0;
        this.outcome = null;
        this.gameState.status = 'playing';
        this.gameState.result = undefined;
        this.refresh();
        break;
      case 'MOVE': {
        const parts = payload.split(' ');
        const moverIdx = ctx.players.indexOf(parts[0]);
        const row = parseInt(parts[1], 10);
        const col = parseInt(parts[2], 10);
        if (moverIdx < 0 || isNaN(row) || isNaN(col)) return;
        this.board[row * 3 + col] = moverIdx === 0 ? 'X' : 'O';
        this.moveCount += 1;
        this.refresh();
        break;
      }
      case 'WINNER': {
        const w = parseInt(payload, 10);
        this.gameState.status = 'finished';
        this.outcome = w === ctx.myIndex ? 'win' : 'lose';
        this.refresh();
        break;
      }
      case 'DRAW':
        this.gameState.status = 'finished';
        this.outcome = 'draw';
        this.refresh();
        break;
    }
  }

  public getStatus(): GameStatus {
    if (this.gameState.status === 'waiting') {
      return { text: 'Waiting for game to start', tone: 'idle' };
    }
    if (this.gameState.status === 'finished') {
      if (this.outcome === 'win') return { text: 'You Win!', tone: 'win' };
      if (this.outcome === 'lose') return { text: 'You Lose', tone: 'lose' };
      return { text: 'Draw', tone: 'draw' };
    }
    const me = this.ctx.myIndex;
    const isMyTurn = me >= 0 && this.moveCount % 2 === me;
    const sym = me === 0 ? 'X' : 'O';
    return isMyTurn
      ? { text: `Your turn`, tone: 'turn', detail: `Playing as ${sym}` }
      : { text: `Opponent's turn`, tone: 'wait', detail: `You are ${sym}` };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished' || !this.outcome) return null;
    const sym = this.ctx.myIndex === 0 ? 'X' : 'O';
    const title =
      this.outcome === 'win' ? 'Victory' : this.outcome === 'lose' ? 'Defeat' : 'Draw';
    return {
      outcome: this.outcome,
      title,
      details: [`You played as ${sym}`, `${this.moveCount} moves played`],
    };
  }

  protected render(): void {
    this.container.removeChildren();

    const me = this.ctx.myIndex;
    const isMyTurn = this.gameState.status === 'playing' && me >= 0 && this.moveCount % 2 === me;

    for (let i = 0; i < 9; i++) {
      const row = Math.floor(i / 3);
      const col = i % 3;
      const x = col * 120 + 20;
      const y = row * 120 + 20;

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
          style: {
            fontSize: 64,
            fill: this.board[i] === 'X' ? COLORS.BLUE : COLORS.RED,
            fontWeight: 'bold',
          },
        });
        text.x = x + 50 - text.width / 2;
        text.y = y + 50 - text.height / 2;
        this.container.addChild(text);
      }
    }
  }

  private makeMove(row: number, col: number): void {
    if (!this.wsConnected || this.gameState.status !== 'playing') return;
    const me = this.ctx.myIndex;
    if (me < 0 || this.moveCount % 2 !== me) return;
    if (this.board[row * 3 + col]) return;
    this.sendMove(`${row} ${col}`);
  }
}
