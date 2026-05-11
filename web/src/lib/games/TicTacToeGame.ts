import { Graphics, Text } from 'pixi.js';
import { BasePixiGame, type GameContext, type GameResult, type GameStatus } from './BasePixiGame';
import { COLORS } from './types';

const DEFAULT_SIZE = 16;
const DEFAULT_CHAIN = 5;
const CANVAS_SIZE = 600;

/** Spec: judge/protocols/tic-tac-toe.md. m,n,k variant — board side and
 *  chain length come from GAME_STARTED payload (`<board_size> <win_chain>`). */
export class TicTacToeGame extends BasePixiGame {
  private boardSize = DEFAULT_SIZE;
  private winChain = DEFAULT_CHAIN;
  private board: ('X' | 'O' | null)[] = [];
  private moveCount = 0;
  private outcome: 'win' | 'lose' | 'draw' | null = null;

  public handleEvent(kind: string, payload: string, ctx: GameContext): void {
    this.ctx = ctx;

    switch (kind) {
      case 'GAME_STARTED': {
        const parts = payload.split(/\s+/);
        const size = parseInt(parts[0], 10);
        const chain = parseInt(parts[1], 10);
        this.boardSize = Number.isFinite(size) && size > 0 ? size : DEFAULT_SIZE;
        this.winChain = Number.isFinite(chain) && chain > 0 ? chain : DEFAULT_CHAIN;
        this.board = Array(this.boardSize * this.boardSize).fill(null);
        this.moveCount = 0;
        this.outcome = null;
        this.gameState.status = 'playing';
        this.gameState.result = undefined;
        this.refresh();
        break;
      }
      case 'MOVE': {
        const parts = payload.split(' ');
        const moverIdx = ctx.players.indexOf(parts[0]);
        const row = parseInt(parts[1], 10);
        const col = parseInt(parts[2], 10);
        if (moverIdx < 0 || isNaN(row) || isNaN(col)) return;
        const idx = row * this.boardSize + col;
        if (idx >= 0 && idx < this.board.length) {
          this.board[idx] = moverIdx === 0 ? 'X' : 'O';
          this.moveCount += 1;
        }
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
    const detail = `Playing as ${sym} · ${this.boardSize}×${this.boardSize}, ${this.winChain} in a row`;
    return isMyTurn
      ? { text: `Your turn`, tone: 'turn', detail }
      : { text: `Opponent's turn`, tone: 'wait', detail };
  }

  public getResult(): GameResult | null {
    if (this.gameState.status !== 'finished' || !this.outcome) return null;
    const sym = this.ctx.myIndex === 0 ? 'X' : 'O';
    const title =
      this.outcome === 'win' ? 'Victory' : this.outcome === 'lose' ? 'Defeat' : 'Draw';
    return {
      outcome: this.outcome,
      title,
      details: [
        `You played as ${sym}`,
        `${this.moveCount} moves played`,
        `Board ${this.boardSize}×${this.boardSize} · ${this.winChain}-in-a-row`,
      ],
    };
  }

  protected render(): void {
    this.container.removeChildren();
    if (this.boardSize === 0) return;

    const me = this.ctx.myIndex;
    const isMyTurn = this.gameState.status === 'playing' && me >= 0 && this.moveCount % 2 === me;

    // Lay out a CANVAS_SIZE px square divided into boardSize cells with a
    // 10 px outer padding. Cells scale with board size; symbols scale to fit.
    const padding = 10;
    const inner = CANVAS_SIZE - padding * 2;
    const cellSize = inner / this.boardSize;
    const symbolFont = Math.max(10, Math.floor(cellSize * 0.6));
    const strokeWidth = this.boardSize <= 8 ? 2 : 1;

    for (let i = 0; i < this.board.length; i++) {
      const row = Math.floor(i / this.boardSize);
      const col = i % this.boardSize;
      const x = padding + col * cellSize;
      const y = padding + row * cellSize;

      const cell = new Graphics();
      cell.rect(x, y, cellSize, cellSize);
      cell.fill(COLORS.WHITE);
      cell.stroke({ width: strokeWidth, color: COLORS.BLACK });

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
            fontSize: symbolFont,
            fill: this.board[i] === 'X' ? COLORS.BLUE : COLORS.RED,
            fontWeight: 'bold',
          },
        });
        text.x = x + cellSize / 2 - text.width / 2;
        text.y = y + cellSize / 2 - text.height / 2;
        this.container.addChild(text);
      }
    }
  }

  private makeMove(row: number, col: number): void {
    if (!this.wsConnected || this.gameState.status !== 'playing') return;
    const me = this.ctx.myIndex;
    if (me < 0 || this.moveCount % 2 !== me) return;
    if (this.board[row * this.boardSize + col]) return;
    this.sendMove(`${row} ${col}`);
  }
}
