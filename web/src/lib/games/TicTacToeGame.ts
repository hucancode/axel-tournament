import { Graphics, Text } from 'pixi.js';
import { BasePixiGame } from './BasePixiGame';
import { COLORS, parseMessage } from './types';

export class TicTacToeGame extends BasePixiGame {
  private board: (string | null)[] = Array(9).fill(null);
  private mySymbol: 'X' | 'O' | null = null;
  private isMyTurn = false;
  private cells: Graphics[] = [];

  protected handleMessage(data: string): void {
    const parts = parseMessage(data);
    if (!parts.length) return;

    switch (parts[0]) {
      case 'START':
        if (parts.length === 2) {
          this.mySymbol = parts[1] as 'X' | 'O';
          this.gameState.status = 'playing';
          this.isMyTurn = this.mySymbol === 'X';
          this.render();
        }
        break;
      case 'MOVE':
        if (parts.length === 3) {
          const row = parseInt(parts[1]);
          const col = parseInt(parts[2]);
          const index = row * 3 + col;
          this.board[index] = this.mySymbol === 'X' ? 'O' : 'X';
          this.isMyTurn = true;
          this.render();
        }
        break;
      case 'WIN':
        this.gameState = { status: 'finished', result: 'You Win!' };
        this.render();
        break;
      case 'LOSE':
        this.gameState = { status: 'finished', result: 'You Lose!' };
        this.render();
        break;
      case 'DRAW':
        this.gameState = { status: 'finished', result: 'Draw!' };
        this.render();
        break;
    }
  }

  protected render(): void {
    this.container.removeChildren();
    
    // Draw board
    for (let i = 0; i < 9; i++) {
      const row = Math.floor(i / 3);
      const col = i % 3;
      const x = col * 120 + 40;
      const y = row * 120 + 80;
      
      const cell = new Graphics();
      cell.rect(x, y, 100, 100);
      cell.fill(COLORS.WHITE);
      cell.stroke({ width: 2, color: COLORS.BLACK });
      
      if (this.gameState.status === 'playing' && !this.board[i] && this.isMyTurn) {
        cell.interactive = true;
        cell.cursor = 'pointer';
        cell.on('pointerdown', () => this.makeMove(row, col));
      }
      
      this.container.addChild(cell);
      this.cells[i] = cell;
      
      // Draw symbol
      if (this.board[i]) {
        const text = new Text({
          text: this.board[i]!,
          style: { fontSize: 48, fill: COLORS.BLACK }
        });
        text.x = x + 50 - text.width / 2;
        text.y = y + 50 - text.height / 2;
        this.container.addChild(text);
      }
    }
    
    // Status text
    const status = new Text({
      text: this.getStatusText(),
      style: { fontSize: 16, fill: COLORS.BLACK }
    });
    status.x = 200 - status.width / 2;
    status.y = 20;
    this.container.addChild(status);
  }

  private makeMove(row: number, col: number): void {
    if (!this.wsConnected || this.gameState.status !== 'playing' || !this.isMyTurn) return;
    
    const index = row * 3 + col;
    if (this.board[index]) return;
    
    this.sendMessage(`MOVE ${row} ${col}`);
    this.board[index] = this.mySymbol;
    this.isMyTurn = false;
    this.render();
  }

  private getStatusText(): string {
    if (this.gameState.status === 'waiting') return 'Waiting for game...';
    if (this.gameState.status === 'finished') return this.gameState.result || 'Game Over';
    return this.isMyTurn ? `Your turn (${this.mySymbol})` : "Opponent's turn";
  }
}
